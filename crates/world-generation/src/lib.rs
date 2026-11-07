//! Muestra pequeña y determinista de elevaciones, independiente de la GPU.
use noise::{NoiseFn, Perlin};

pub struct Terrain {
    noise: Perlin,
}

impl Terrain {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    pub fn height(&self, x: i32, y: i32) -> f32 {
        (self.noise.get([x as f64 * 0.13, y as f64 * 0.13]) as f32 * 2.5).clamp(-3.0, 3.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_reproduces_terrain_and_changes_the_world() {
        let a = Terrain::new(7);
        let b = Terrain::new(7);
        let c = Terrain::new(8);
        let mut differs = false;
        for x in -10..10 {
            for y in -10..10 {
                assert_eq!(a.height(x, y), b.height(x, y));
                assert!(a.height(x, y).is_finite());
                assert!((-3.0..=3.0).contains(&a.height(x, y)));
                differs |= a.height(x, y) != c.height(x, y);
            }
        }
        assert!(differs);
    }
}
