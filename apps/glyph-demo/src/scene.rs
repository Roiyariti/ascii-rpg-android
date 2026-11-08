use ascii_renderer::{Glyph, MAX_GLYPHS};
use game_core::{GameState, IsoCamera};
use glam::{IVec2, Vec2, Vec3};
use std::time::Instant;
use turn_system::Action;
use world_generation::{CHUNK_SIZE, ChunkWorld, Landform, Tile};

pub fn viewport(width: u32, height: u32) -> Vec2 {
    Vec2::new(width.max(1) as f32 / height.max(1) as f32 * 600.0, 600.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    Action(Action),
    Zoom(f32),
    Center,
    Debug,
    Seed,
}

#[derive(Default)]
pub struct Stats {
    pub tiles: usize,
    pub chunks: usize,
    pub cache_bytes: usize,
    pub detail: usize,
    pub cpu_ms: f32,
    pub generated: usize,
}

#[derive(Clone, Copy)]
struct VisibleTile {
    tile: Tile,
    right: Tile,
    front: Tile,
    position: Vec3,
    distance: f32,
}

pub struct Scene {
    world: ChunkWorld,
    seed: u32,
    anchor: IVec2,
    visual: Vec3,
    from: Vec3,
    elapsed: f32,
    pub camera: IsoCamera,
    pub debug: bool,
    visible: Vec<VisibleTile>,
    pub stats: Stats,
}

impl Scene {
    pub fn new(state: &GameState) -> Self {
        let world = ChunkWorld::new(state.seed);
        let visual = Vec3::new(
            0.5,
            0.5,
            world.terrain.height(state.player.x, state.player.y),
        );
        Self {
            world,
            seed: state.seed,
            anchor: state.player,
            visual,
            from: visual,
            elapsed: 1.0,
            camera: IsoCamera::new(Vec2::new(960.0, 600.0)),
            debug: false,
            visible: Vec::with_capacity(4096),
            stats: Stats::default(),
        }
    }

    pub fn zoom(&mut self, factor: f32) {
        self.camera.zoom = (self.camera.zoom * factor).clamp(0.85, 2.0);
    }

    pub fn build(
        &mut self,
        out: &mut Vec<Glyph>,
        state: &GameState,
        time: f32,
        dt: f32,
        viewport: Vec2,
    ) {
        let started = Instant::now();
        if self.seed != state.seed {
            self.world = ChunkWorld::new(state.seed);
            self.seed = state.seed;
            self.visual.z = self.world.terrain.height(state.player.x, state.player.y);
            self.from = self.visual;
        }
        if self.anchor != state.player {
            self.visual.x += (i64::from(self.anchor.x) - i64::from(state.player.x)) as f32;
            self.visual.y += (i64::from(self.anchor.y) - i64::from(state.player.y)) as f32;
            self.anchor = state.player;
            self.from = self.visual;
            self.elapsed = 0.0;
        }
        self.elapsed = (self.elapsed + dt.max(0.0)).min(1.0);
        let t = (self.elapsed / 0.18).min(1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        let target = Vec3::new(
            0.5,
            0.5,
            self.world.terrain.height(state.player.x, state.player.y),
        );
        self.visual = self.from.lerp(target, smooth);
        self.camera.target = self.visual;
        self.camera.viewport = viewport;
        // Incluye el relieve y el pan; el máximo mantiene acotada la caché.
        let radius = (viewport.x / (4.0 * self.camera.tile_half.x * self.camera.zoom)
            + viewport.y / (4.0 * self.camera.tile_half.y * self.camera.zoom)
            + 12.0
            + self.camera.pan.length() / 28.0)
            .ceil()
            .clamp(16.0, 48.0) as i32;
        self.world.update(state.player.to_array(), radius);
        self.visible.clear();
        for chunk in self.world.chunks() {
            let [ox, oy] = chunk.coord.origin();
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let tile = chunk.tile(x, y);
                    let position = Vec3::new(
                        (ox + i64::from(x) - i64::from(state.player.x)) as f32,
                        (oy + i64::from(y) - i64::from(state.player.y)) as f32,
                        tile.height,
                    );
                    let projected = self.camera.project(position + Vec3::new(0.5, 0.5, 0.0));
                    let margin = 70.0 * self.camera.zoom;
                    if projected.x < -margin
                        || projected.x > viewport.x + margin
                        || projected.y < -margin
                        || projected.y > viewport.y + 150.0 * self.camera.zoom
                    {
                        continue;
                    }
                    self.visible.push(VisibleTile {
                        tile,
                        right: chunk.tile(x + 1, y),
                        front: chunk.tile(x, y + 1),
                        position,
                        distance: position.truncate().distance(self.visual.truncate()),
                    });
                }
            }
        }
        // Prioridad cercana si una vista extrema agota incluso el detalle mínimo.
        self.visible.sort_unstable_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.position.x.total_cmp(&b.position.x))
                .then(a.position.y.total_cmp(&b.position.y))
        });
        // Reducir subdivisiones antes de escribir fuera del presupuesto CPU/GPU.
        let mut detail = 3;
        let mut rendered;
        loop {
            out.clear();
            let mut exceeded = false;
            rendered = 0;
            for tile in &self.visible {
                draw_tile(out, &self.camera, tile, detail);
                rendered += 1;
                if out.len() > MAX_GLYPHS - 512 {
                    exceeded = true;
                    break;
                }
            }
            if !exceeded || detail == 1 {
                break;
            }
            detail -= 1;
        }
        hero(out, &self.camera, self.visual, time);
        self.stats = Stats {
            tiles: rendered,
            chunks: self.world.chunks().count(),
            cache_bytes: self.world.bytes(),
            detail,
            cpu_ms: started.elapsed().as_secs_f32() * 1000.0,
            generated: self.world.generated(),
        };
        self.hud(out, state, viewport, dt);
    }

    fn hud(&self, out: &mut Vec<Glyph>, state: &GameState, viewport: Vec2, dt: f32) {
        let bone = [0.70, 0.66, 0.53, 1.0];
        panel(out, Vec2::new(12.0, 8.0), Vec2::new(360.0, 40.0), viewport);
        panel(
            out,
            Vec2::new(16.0, viewport.y - 116.0),
            Vec2::new(158.0, 108.0),
            viewport,
        );
        panel(
            out,
            Vec2::new(viewport.x - 282.0, viewport.y - 155.0),
            Vec2::new(258.0, 148.0),
            viewport,
        );
        text(
            out,
            &format!("UMBRAL / TERRENO   SEMILLA {}", state.seed),
            Vec2::new(20.0, 16.0),
            9.0,
            viewport,
            bone,
        );
        text(
            out,
            &format!(
                "T {:04}  X {} Y {} Z {:.1}",
                state.clock.current(),
                state.player.x,
                state.player.y,
                self.world.terrain.height(state.player.x, state.player.y)
            ),
            Vec2::new(20.0, 32.0),
            9.0,
            viewport,
            bone,
        );
        for (label, at, _) in buttons(viewport) {
            text(out, label, at, 12.0, viewport, [0.80, 0.49, 0.20, 1.0]);
        }
        if self.debug {
            panel(
                out,
                Vec2::new((viewport.x - 390.0).max(12.0), 8.0),
                Vec2::new(378.0, 112.0),
                viewport,
            );
            let information = format!(
                "FPS {:.0}  CPU ESCENA {:.2}ms\n{} GLIFOS  {} CELDAS  LOD {}\n{} CHUNKS  {} KiB CPU\nGPU INST 1280 KiB  Z16\nTAB DEBUG  IJKL PAN  HOME CENTRO",
                1.0 / dt.max(0.001),
                self.stats.cpu_ms,
                out.len(),
                self.stats.tiles,
                self.stats.detail,
                self.stats.chunks,
                self.stats.cache_bytes / 1024
            );
            text(
                out,
                &information,
                Vec2::new((viewport.x - 380.0).max(20.0), 18.0),
                10.0,
                viewport,
                bone,
            );
            text(
                out,
                "[SEED]",
                Vec2::new(viewport.x - 110.0, 92.0),
                12.0,
                viewport,
                bone,
            );
        }
    }
}

fn panel(out: &mut Vec<Glyph>, at: Vec2, size: Vec2, viewport: Vec2) {
    out.push(
        Glyph::new(
            ' ',
            at.to_array(),
            size.to_array(),
            viewport.to_array(),
            [0.012, 0.018, 0.021, 1.0],
        )
        .with_background(1.0),
    );
}

fn text(
    out: &mut Vec<Glyph>,
    value: &str,
    origin: Vec2,
    cell: f32,
    viewport: Vec2,
    color: [f32; 4],
) {
    for (y, line) in value.lines().enumerate() {
        for (x, character) in line.chars().enumerate() {
            if character != ' ' {
                out.push(Glyph::new(
                    character,
                    (origin + Vec2::new(x as f32 * cell, y as f32 * (cell + 4.0))).to_array(),
                    [cell; 2],
                    viewport.to_array(),
                    color,
                ));
            }
        }
    }
}

fn top_color(tile: Tile) -> [f32; 4] {
    let base = match tile.landform {
        Landform::Mountain => [0.36, 0.35, 0.31],
        Landform::Ravine => [0.20, 0.27, 0.24],
        Landform::Plateau => [0.37, 0.31, 0.21],
        Landform::Plain => [0.28, 0.36, 0.20],
        Landform::Hills => [0.28, 0.33, 0.22],
    };
    let variation = 0.88 + (tile.variation & 255) as f32 / 255.0 * 0.20;
    [
        base[0] * variation,
        base[1] * variation,
        base[2] * variation,
        1.0,
    ]
}

fn surface(
    out: &mut Vec<Glyph>,
    camera: &IsoCamera,
    character: char,
    origin: Vec3,
    u: Vec3,
    v: Vec3,
    color: [f32; 4],
) {
    out.push(
        Glyph::quad(
            character,
            camera.project(origin).to_array(),
            camera.project_vector(u).to_array(),
            camera.project_vector(v).to_array(),
            camera.viewport.to_array(),
            color,
        )
        .with_background(0.28),
    );
}

fn draw_tile(out: &mut Vec<Glyph>, camera: &IsoCamera, visible: &VisibleTile, detail: usize) {
    let tile = visible.tile;
    let n = if visible.distance < 9.0 {
        3
    } else if visible.distance > 15.0 {
        detail.min(2)
    } else {
        detail
    };
    let step = 1.0 / n as f32;
    let origin = Vec3::new(
        visible.position.x,
        visible.position.y,
        tile.corner_height(0.0, 0.0),
    );
    let color = top_color(tile);
    let characters: &[char] = match tile.landform {
        Landform::Mountain | Landform::Plateau => &[':', ';', '+', '.', '#', ':'],
        _ => &[',', '\'', '.', ';', '"', '.'],
    };
    let u = Vec3::new(step, 0.0, tile.gradient[0] * step);
    let v = Vec3::new(0.0, step, tile.gradient[1] * step);
    for y in 0..n {
        for x in 0..n {
            let index =
                (tile.variation.rotate_right(((x + y * n) * 3) as u32) as usize) % characters.len();
            surface(
                out,
                camera,
                characters[index],
                origin + u * x as f32 + v * y as f32,
                u,
                v,
                color,
            );
        }
    }
    // Caras hacia +x/+y. Se prolongan hasta el vecino: nada depende del borde del chunk.
    for (side, neighbor) in [(0, visible.right), (1, visible.front)] {
        let (start, edge, own, other) = if side == 0 {
            (
                Vec3::new(
                    visible.position.x + 1.0,
                    visible.position.y,
                    tile.corner_height(1.0, 0.0),
                ),
                Vec3::new(0.0, 1.0, tile.gradient[1]),
                [tile.corner_height(1.0, 0.0), tile.corner_height(1.0, 1.0)],
                [
                    neighbor.corner_height(0.0, 0.0),
                    neighbor.corner_height(0.0, 1.0),
                ],
            )
        } else {
            (
                Vec3::new(
                    visible.position.x,
                    visible.position.y + 1.0,
                    tile.corner_height(0.0, 1.0),
                ),
                Vec3::new(1.0, 0.0, tile.gradient[0]),
                [tile.corner_height(0.0, 1.0), tile.corner_height(1.0, 1.0)],
                [
                    neighbor.corner_height(0.0, 0.0),
                    neighbor.corner_height(1.0, 0.0),
                ],
            )
        };
        let drop = (own[0] - other[0]).max(own[1] - other[1]).max(0.0);
        if drop < 0.015 {
            continue;
        }
        let rows = if detail == 1 {
            1
        } else {
            (drop * 2.4).ceil().clamp(1.0, 8.0) as usize
        };
        let shade = if side == 0 { 0.57 } else { 0.73 };
        let wall_color = [
            color[0] * shade,
            color[1] * shade * 0.88,
            color[2] * shade * 0.85,
            1.0,
        ];
        let u = edge / n as f32;
        let v = Vec3::new(0.0, 0.0, -drop / rows as f32);
        for row in 0..rows {
            for column in 0..n {
                let character = if (row + column + tile.variation as usize).is_multiple_of(3) {
                    '#'
                } else {
                    ':'
                };
                surface(
                    out,
                    camera,
                    character,
                    start + u * column as f32 + v * row as f32,
                    u,
                    v,
                    wall_color,
                );
            }
        }
    }
}

fn hero(out: &mut Vec<Glyph>, camera: &IsoCamera, position: Vec3, time: f32) {
    let rows = [
        "   /^^\\   ",
        "  /####\\  ",
        " /##==##\\ ",
        " |(.)(.)| ",
        "  \\_v_/  ",
        " /[+##+]\\ ",
        "<|##++##|>",
        " /|/||\\|\\",
        "  d/  \\b ",
    ];
    let feet = camera.project(position);
    let zoom = camera.zoom;
    let origin = feet.truncate() + Vec2::new(-24.0, -64.0 + (time * 2.0).sin() * 0.5) * zoom;
    for (y, row) in rows.iter().enumerate() {
        for (x, character) in row.chars().enumerate() {
            if character == ' ' {
                continue;
            }
            let color = match character {
                '#' | '^' => [0.22, 0.10, 0.16, 1.0],
                '(' | ')' | '.' | 'v' | '_' => [0.88, 0.63, 0.38, 1.0],
                '+' | '=' | '[' | ']' => [0.46, 0.64, 0.70, 1.0],
                _ => [0.72, 0.65, 0.49, 1.0],
            };
            let at = origin + Vec2::new(x as f32 * 5.0, y as f32 * 7.0) * zoom;
            let height = ((feet.y - at.y) / (camera.height_scale * zoom)).max(0.0);
            let depth = feet.z + camera.project_vector(Vec3::Z * height).z - 0.0001;
            out.push(
                Glyph::new(
                    character,
                    at.to_array(),
                    [6.0 * zoom; 2],
                    camera.viewport.to_array(),
                    color,
                )
                .with_depth(depth),
            );
        }
    }
}

fn buttons(viewport: Vec2) -> [(&'static str, Vec2, Control); 12] {
    let h = viewport.y;
    let w = viewport.x;
    [
        (
            "[<]",
            Vec2::new(24.0, h - 62.0),
            Control::Action(Action::Move { dx: -1, dy: 0 }),
        ),
        (
            "[^]",
            Vec2::new(78.0, h - 104.0),
            Control::Action(Action::Move { dx: 0, dy: -1 }),
        ),
        (
            "[v]",
            Vec2::new(78.0, h - 32.0),
            Control::Action(Action::Move { dx: 0, dy: 1 }),
        ),
        (
            "[>]",
            Vec2::new(132.0, h - 62.0),
            Control::Action(Action::Move { dx: 1, dy: 0 }),
        ),
        (
            "[WAIT]",
            Vec2::new(w - 270.0, h - 62.0),
            Control::Action(Action::Wait),
        ),
        (
            "[HIT]",
            Vec2::new(w - 160.0, h - 62.0),
            Control::Action(Action::Attack),
        ),
        (
            "[ARMA]",
            Vec2::new(w - 270.0, h - 28.0),
            Control::Action(Action::SwitchWeapon),
        ),
        (
            "[EQUIP]",
            Vec2::new(w - 160.0, h - 28.0),
            Control::Action(Action::Equip),
        ),
        (
            "[-]",
            Vec2::new(w - 192.0, h - 108.0),
            Control::Zoom(1.0 / 1.15),
        ),
        ("[+]", Vec2::new(w - 136.0, h - 108.0), Control::Zoom(1.15)),
        ("[O]", Vec2::new(w - 80.0, h - 108.0), Control::Center),
        ("[DBG]", Vec2::new(w - 120.0, h - 145.0), Control::Debug),
    ]
}

pub fn control_at(point: Vec2, viewport: Vec2, debug: bool) -> Option<Control> {
    if debug
        && point.x >= viewport.x - 115.0
        && point.x <= viewport.x - 28.0
        && (84.0..=116.0).contains(&point.y)
    {
        return Some(Control::Seed);
    }
    buttons(viewport)
        .into_iter()
        .find_map(|(label, at, control)| {
            (point.x >= at.x - 8.0
                && point.x <= at.x + label.len() as f32 * 12.0 + 8.0
                && point.y >= at.y - 8.0
                && point.y <= at.y + 24.0)
                .then_some(control)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn render_does_not_tick_and_cache_survives_camera_changes() {
        let mut state = GameState::default();
        let mut scene = Scene::new(&state);
        let mut out = Vec::with_capacity(MAX_GLYPHS);
        let viewport = Vec2::new(1333.0, 600.0);
        scene.build(&mut out, &state, 0.0, 0.02, viewport);
        assert!(out.len() > 2000 && out.len() <= MAX_GLYPHS);
        let generated = scene.stats.generated;
        scene.build(&mut out, &state, 10.0, 0.02, viewport);
        assert_eq!(state.clock.current(), 0);
        assert_eq!(scene.stats.generated, generated);
        assert!(state.apply(Action::Move { dx: 1, dy: 0 }));
        scene.build(&mut out, &state, 10.02, 0.02, viewport);
        assert!(
            scene.visual.x < 0.5,
            "La posición visual aún interpola el paso lógico"
        );
        scene.build(&mut out, &state, 10.22, 0.2, viewport);
        assert!((scene.visual.x - 0.5).abs() < 0.001);
        assert_eq!(state.clock.current(), 1);
        scene.zoom(0.01);
        scene.build(&mut out, &state, 10.24, 0.02, Vec2::new(2133.0, 600.0));
        assert!(
            out.len() <= MAX_GLYPHS,
            "La relación 32:9 y zoom mínimo deben caber"
        );
        assert!(scene.stats.chunks <= 49);
        assert!(
            out.capacity() <= MAX_GLYPHS,
            "El ajuste de detalle no debe ampliar el buffer CPU"
        );
    }
    #[test]
    fn touch_regions_follow_the_aspect_ratio_and_view_controls_do_not_tick() {
        for width in [960.0, 1333.0, 2133.0] {
            let viewport = Vec2::new(width, 600.0);
            assert_eq!(
                control_at(Vec2::new(84.0, 500.0), viewport, false),
                Some(Control::Action(Action::Move { dx: 0, dy: -1 }))
            );
            assert_eq!(
                control_at(Vec2::new(width - 130.0, 496.0), viewport, false),
                Some(Control::Zoom(1.15))
            );
            assert_eq!(
                control_at(Vec2::new(width / 2.0, 300.0), viewport, false),
                None
            );
        }
    }
}
