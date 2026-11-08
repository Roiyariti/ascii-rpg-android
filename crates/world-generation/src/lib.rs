//! Terreno determinista en coordenadas absolutas; sin dependencias de GPU.
mod chunks;
mod height;
pub use chunks::{CHUNK_SIZE, Chunk, ChunkCoord, ChunkWorld, MAX_VIEW_RADIUS};
use height::HeightField;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landform {
    Plain,
    Hills,
    Plateau,
    Mountain,
    Ravine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    pub height: f32,
    /// Inclinación limitada; las diferencias restantes forman escarpes.
    pub gradient: [f32; 2],
    pub landform: Landform,
    pub variation: u32,
}

impl Tile {
    pub fn corner_height(self, u: f32, v: f32) -> f32 {
        self.height + self.gradient[0] * (u - 0.5) + self.gradient[1] * (v - 0.5)
    }
}

pub struct Terrain {
    seed: u32,
    field: HeightField,
}

impl Terrain {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            field: HeightField::new(seed),
        }
    }

    pub fn height(&self, x: i32, y: i32) -> f32 {
        self.elevation(i64::from(x), i64::from(y)).0
    }

    pub fn tile(&self, x: i64, y: i64) -> Tile {
        let (height, landform) = self.elevation(x, y);
        let dx = (self.elevation(x + 1, y).0 - self.elevation(x - 1, y).0) * 0.5;
        let dy = (self.elevation(x, y + 1).0 - self.elevation(x, y - 1).0) * 0.5;
        self.make_tile(x, y, height, landform, [dx, dy])
    }

    fn elevation(&self, x: i64, y: i64) -> (f32, Landform) {
        self.field.sample(x as f64, y as f64)
    }

    fn make_tile(
        &self,
        x: i64,
        y: i64,
        height: f32,
        landform: Landform,
        gradient: [f32; 2],
    ) -> Tile {
        Tile {
            height,
            gradient: gradient.map(|v| v.clamp(-0.32, 0.32)),
            landform,
            variation: coordinate_hash(self.seed, x, y),
        }
    }
}

/// Hash estable, sin RNG global ni dependencia del orden de visita.
pub fn coordinate_hash(seed: u32, x: i64, y: i64) -> u32 {
    let mut value = (x as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (y as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ u64::from(seed);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (value ^ (value >> 31)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seed_reproduces_a_finite_varied_landscape() {
        let (a, b, c) = (Terrain::new(42), Terrain::new(42), Terrain::new(73));
        let (mut low, mut high, mut differences) = (f32::MAX, f32::MIN, 0);
        for x in (-80..=80).step_by(5) {
            for y in (-80..=80).step_by(5) {
                let tile = a.tile(x, y);
                assert_eq!(tile, b.tile(x, y));
                assert!(tile.height.is_finite() && (-10.0..=18.0).contains(&tile.height));
                assert!(
                    tile.gradient
                        .iter()
                        .all(|v| v.is_finite() && v.abs() <= 0.32)
                );
                low = low.min(tile.height);
                high = high.max(tile.height);
                differences += usize::from(tile != c.tile(x, y));
            }
        }
        assert!(
            high - low > 7.0,
            "El paisaje debe tener relieve: {low}..{high}"
        );
        assert!(differences > 500);
    }
}
