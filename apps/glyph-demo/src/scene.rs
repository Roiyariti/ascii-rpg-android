use ascii_renderer::Glyph;
use game_core::{GameState, project};
use glam::{Vec2, Vec3};
use turn_system::Action;
use world_generation::Terrain;

pub const CANVAS: [f32; 2] = [960.0, 600.0];
const BONE: [f32; 4] = [0.68, 0.62, 0.46, 1.0];
const DIM: [f32; 4] = [0.20, 0.29, 0.30, 1.0];
const GOLD: [f32; 4] = [0.87, 0.44, 0.13, 1.0];

fn text(out: &mut Vec<Glyph>, text: &str, origin: Vec2, cell: f32, color: [f32; 4]) {
    for (y, line) in text.lines().enumerate() {
        for (x, character) in line.chars().enumerate() {
            if character != ' ' {
                out.push(Glyph::new(
                    character,
                    (origin + Vec2::new(x as f32 * cell, y as f32 * cell)).to_array(),
                    [cell; 2],
                    CANVAS,
                    color,
                ));
            }
        }
    }
}

fn hero(out: &mut Vec<Glyph>, center: Vec2, time: f32) {
    let origin = center + Vec2::new(-24.0, -64.0 + (time * 2.0).sin() * 1.5);
    // Cada material conserva glifos propios: cabello, rostro, metal y tela.
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
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == ' ' {
                continue;
            }
            let color = match c {
                '#' | '^' => [0.23, 0.12, 0.20, 1.0],
                '(' | ')' | '.' | 'v' | '_' => [0.88, 0.63, 0.38, 1.0],
                '+' | '=' | '[' | ']' => [0.42, 0.60, 0.64, 1.0],
                _ => BONE,
            };
            text(
                out,
                &c.to_string(),
                origin + Vec2::new(x as f32 * 5.0, y as f32 * 7.0),
                6.0,
                color,
            );
        }
    }
}

pub fn build(out: &mut Vec<Glyph>, state: &GameState, time: f32) {
    out.clear();
    text(
        out,
        "A S C I I   R P G   2 . 5 D",
        Vec2::new(28.0, 24.0),
        17.0,
        BONE,
    );
    text(
        out,
        "EL UMBRAL DE CENIZA  /  PROTOTIPO GPU",
        Vec2::new(30.0, 54.0),
        9.0,
        DIM,
    );
    text(
        out,
        &format!("TURNO {:04}", state.clock.current()),
        Vec2::new(760.0, 30.0),
        13.0,
        GOLD,
    );

    let terrain = Terrain::new(state.seed);
    let camera = project(state.player_position());
    let origin = Vec2::new(480.0, 310.0);
    // Orden del suelo por diagonales; las entidades son capas posteriores en esta demo.
    for diagonal in -16..=16 {
        for dx in -8..=8 {
            let dy = diagonal - dx;
            if !(-8..=8).contains(&dy) {
                continue;
            }
            let x = state.player.x.saturating_add(dx);
            let y = state.player.y.saturating_add(dy);
            let height = terrain.height(x, y);
            let point = origin + project(Vec3::new(x as f32, y as f32, height)) - camera;
            let tone = (0.12 + (height + 3.0) * 0.025).clamp(0.1, 0.3);
            let color = [tone * 0.8, tone, tone * 0.83, 1.0];
            text(
                out,
                if height < -0.7 { "~~" } else { "/..\\" },
                point,
                7.0,
                if height < -0.7 {
                    [0.07, 0.24, 0.32, 1.0]
                } else {
                    color
                },
            );
            if height > 0.6 {
                text(out, ": :", point + Vec2::new(3.0, 7.0), 6.0, DIM);
            }
            if x == -5 && y == 1 {
                text(
                    out,
                    "   ^\n  /|\\\n /#|#\\\n/##|##\\\n   |",
                    point + Vec2::new(-15.0, -46.0),
                    8.0,
                    [0.12, 0.30, 0.22, 1.0],
                );
            }
            if x == 4 && y == -3 {
                text(
                    out,
                    "+-+   +-+\n|#|   |#|\n|#|___|#|\n|#/#.#\\#|\n+-------+",
                    point + Vec2::new(-30.0, -50.0),
                    8.0,
                    [0.29, 0.31, 0.34, 1.0],
                );
            }
        }
    }
    hero(out, origin, time);
    let pulse = 0.5 + (time * 4.0).sin() * 0.2;
    text(
        out,
        "*\n|",
        origin + Vec2::new(37.0, -48.0),
        8.0,
        [0.95, pulse, 0.08, 1.0],
    );
    text(
        out,
        "WASD / FLECHAS: MOVER   ESPACIO: ESPERAR",
        Vec2::new(28.0, 464.0),
        9.0,
        BONE,
    );
    text(
        out,
        "F: ATAQUE   Q: ARMA   E: EQUIPAR   ESC: SALIR",
        Vec2::new(28.0, 480.0),
        9.0,
        DIM,
    );

    for (label, x, y) in buttons() {
        text(out, label, Vec2::new(x, y), 12.0, GOLD);
    }
    text(out, "ATLAS ASCII 32-126", Vec2::new(570.0, 512.0), 8.0, DIM);
    for row in 0..3 {
        let sample: String = (32 + row * 32..(64 + row * 32).min(127))
            .map(|c| char::from_u32(c).unwrap())
            .collect();
        text(
            out,
            &sample,
            Vec2::new(570.0, 529.0 + row as f32 * 15.0),
            10.0,
            BONE,
        );
    }
}

fn buttons() -> [(&'static str, f32, f32); 8] {
    [
        ("[<]", 28.0, 545.0),
        ("[^]", 82.0, 515.0),
        ("[v]", 82.0, 560.0),
        ("[>]", 136.0, 545.0),
        ("[WAIT]", 205.0, 535.0),
        ("[HIT]", 300.0, 535.0),
        ("[ARMA]", 205.0, 565.0),
        ("[EQUIP]", 300.0, 565.0),
    ]
}

pub fn action_at(position: Vec2) -> Option<Action> {
    let actions = [
        Action::Move { dx: -1, dy: 0 },
        Action::Move { dx: 0, dy: -1 },
        Action::Move { dx: 0, dy: 1 },
        Action::Move { dx: 1, dy: 0 },
        Action::Wait,
        Action::Attack,
        Action::SwitchWeapon,
        Action::Equip,
    ];
    buttons()
        .into_iter()
        .zip(actions)
        .find_map(|((label, x, y), action)| {
            (position.x >= x - 5.0
                && position.x <= x + label.len() as f32 * 12.0 + 5.0
                && position.y >= y - 5.0
                && position.y <= y + 18.0)
                .then_some(action)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_animation_never_advances_turns_and_stays_in_budget() {
        let state = GameState::default();
        let mut glyphs = Vec::new();
        build(&mut glyphs, &state, 0.0);
        assert!(glyphs.len() > 1000 && glyphs.len() < ascii_renderer::MAX_GLYPHS);
        build(&mut glyphs, &state, 50.0);
        assert_eq!(state.clock.current(), 0);
        assert_eq!(
            action_at(Vec2::new(90.0, 520.0)),
            Some(Action::Move { dx: 0, dy: -1 })
        );
        assert_eq!(action_at(Vec2::new(0.0, 0.0)), None);
    }
}
