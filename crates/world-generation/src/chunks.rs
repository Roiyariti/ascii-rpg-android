use crate::{Terrain, Tile};
use std::collections::BTreeMap;

pub const CHUNK_SIZE: i32 = 16;
pub const MAX_VIEW_RADIUS: i32 = 48;
const PADDED: usize = CHUNK_SIZE as usize + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    pub fn at(x: i32, y: i32) -> Self {
        Self {
            x: x.div_euclid(CHUNK_SIZE),
            y: y.div_euclid(CHUNK_SIZE),
        }
    }
    pub fn origin(self) -> [i64; 2] {
        [
            i64::from(self.x) * i64::from(CHUNK_SIZE),
            i64::from(self.y) * i64::from(CHUNK_SIZE),
        ]
    }
}

pub struct Chunk {
    pub coord: ChunkCoord,
    tiles: Vec<Tile>,
}

impl Chunk {
    pub fn generate(terrain: &Terrain, coord: ChunkCoord) -> Self {
        let [ox, oy] = coord.origin();
        let side = CHUNK_SIZE as usize + 4;
        let mut elevations = Vec::with_capacity(side * side);
        for y in -2..CHUNK_SIZE + 2 {
            for x in -2..CHUNK_SIZE + 2 {
                elevations.push(terrain.elevation(ox + i64::from(x), oy + i64::from(y)));
            }
        }
        let mut tiles = Vec::with_capacity(PADDED * PADDED);
        for y in 1..side - 1 {
            for x in 1..side - 1 {
                let (height, landform) = elevations[y * side + x];
                let gradient = [
                    (elevations[y * side + x + 1].0 - elevations[y * side + x - 1].0) * 0.5,
                    (elevations[(y + 1) * side + x].0 - elevations[(y - 1) * side + x].0) * 0.5,
                ];
                tiles.push(terrain.make_tile(
                    ox + x as i64 - 2,
                    oy + y as i64 - 2,
                    height,
                    landform,
                    gradient,
                ));
            }
        }
        Self { coord, tiles }
    }
    /// Halo -1..=16: los bordes no necesitan que el vecino esté cargado.
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        assert!((-1..=CHUNK_SIZE).contains(&x) && (-1..=CHUNK_SIZE).contains(&y));
        self.tiles[(y + 1) as usize * PADDED + (x + 1) as usize]
    }
    pub fn bytes(&self) -> usize {
        self.tiles.len() * size_of::<Tile>()
    }
}

pub struct ChunkWorld {
    pub terrain: Terrain,
    chunks: BTreeMap<ChunkCoord, Chunk>,
    generated: usize,
}

impl ChunkWorld {
    pub fn new(seed: u32) -> Self {
        Self {
            terrain: Terrain::new(seed),
            chunks: BTreeMap::new(),
            generated: 0,
        }
    }
    /// Ventana acotada; la expulsión no cambia el contenido al regenerarlo.
    pub fn update(&mut self, center: [i32; 2], radius: i32) {
        let radius = radius.clamp(0, MAX_VIEW_RADIUS);
        let first = ChunkCoord::at(
            center[0].saturating_sub(radius),
            center[1].saturating_sub(radius),
        );
        let last = ChunkCoord::at(
            center[0].saturating_add(radius),
            center[1].saturating_add(radius),
        );
        self.chunks.retain(|coord, _| {
            (first.x..=last.x).contains(&coord.x) && (first.y..=last.y).contains(&coord.y)
        });
        for y in first.y..=last.y {
            for x in first.x..=last.x {
                let coord = ChunkCoord { x, y };
                if let std::collections::btree_map::Entry::Vacant(entry) = self.chunks.entry(coord)
                {
                    entry.insert(Chunk::generate(&self.terrain, coord));
                    self.generated += 1;
                }
            }
        }
    }
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }
    pub fn generated(&self) -> usize {
        self.generated
    }
    pub fn bytes(&self) -> usize {
        self.chunks.values().map(Chunk::bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn neighboring_halos_match_direct_sampling_in_any_load_order() {
        for seed in [0, 42, u32::MAX] {
            let terrain = Terrain::new(seed);
            for coord in [
                ChunkCoord { x: -1, y: -1 },
                ChunkCoord { x: 0, y: 0 },
                ChunkCoord { x: 8125, y: -9137 },
            ] {
                let right = Chunk::generate(
                    &terrain,
                    ChunkCoord {
                        x: coord.x + 1,
                        y: coord.y,
                    },
                );
                let chunk = Chunk::generate(&terrain, coord);
                let below = Chunk::generate(
                    &terrain,
                    ChunkCoord {
                        x: coord.x,
                        y: coord.y + 1,
                    },
                );
                let [ox, oy] = coord.origin();
                for offset in -1..=CHUNK_SIZE {
                    assert_eq!(chunk.tile(CHUNK_SIZE, offset), right.tile(0, offset));
                    assert_eq!(chunk.tile(CHUNK_SIZE - 1, offset), right.tile(-1, offset));
                    assert_eq!(chunk.tile(offset, CHUNK_SIZE), below.tile(offset, 0));
                    assert_eq!(chunk.tile(offset, CHUNK_SIZE - 1), below.tile(offset, -1));
                    assert_eq!(
                        chunk.tile(0, offset),
                        terrain.tile(ox, oy + i64::from(offset))
                    );
                }
            }
        }
        assert_eq!(ChunkCoord::at(-1, -17), ChunkCoord { x: -1, y: -2 });
    }
    #[test]
    fn cache_reuses_chunks_and_eviction_preserves_the_seed() {
        let mut world = ChunkWorld::new(42);
        world.update([0, 0], 16);
        let expected = world.chunks().next().unwrap().tile(0, 0);
        let count = world.generated();
        world.update([0, 0], 16);
        assert_eq!(world.generated(), count);
        assert!(world.chunks().count() <= 9);
        world.update([1600, -1600], 16);
        world.update([0, 0], 16);
        assert_eq!(world.chunks().next().unwrap().tile(0, 0), expected);
        assert!(world.chunks().count() <= 9);
    }
}
