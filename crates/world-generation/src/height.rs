use crate::Landform;
use noise::{NoiseFn, Perlin};

pub(super) struct HeightField {
    continent: Perlin,
    detail: Perlin,
    warp_x: Perlin,
    warp_y: Perlin,
    ridge: Perlin,
}

fn fractal(noise: &Perlin, x: f64, y: f64, octaves: u32) -> f64 {
    let (mut value, mut weight, mut frequency, mut total) = (0.0, 1.0, 1.0, 0.0);
    for _ in 0..octaves {
        value += noise.get([x * frequency + 13.71, y * frequency - 41.23]) * weight;
        total += weight;
        weight *= 0.5;
        frequency *= 2.07;
    }
    value / total
}

fn smooth(low: f64, high: f64, value: f64) -> f64 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl HeightField {
    pub fn new(seed: u32) -> Self {
        Self {
            continent: Perlin::new(seed),
            detail: Perlin::new(seed ^ 0x43b0_d7e5),
            warp_x: Perlin::new(seed ^ 0x91e1_0da5),
            warp_y: Perlin::new(seed ^ 0x6a09_e667),
            ridge: Perlin::new(seed ^ 0xbb67_ae85),
        }
    }

    pub fn sample(&self, x: f64, y: f64) -> (f32, Landform) {
        let wx = x + fractal(&self.warp_x, x * 0.016, y * 0.016, 3) * 17.0;
        let wy = y + fractal(&self.warp_y, x * 0.016, y * 0.016, 3) * 17.0;
        let continent = fractal(&self.continent, wx * 0.008, wy * 0.008, 5);
        let hills = fractal(&self.detail, wx * 0.037, wy * 0.037, 4);
        let region = self.continent.get([wx * 0.004 + 81.3, wy * 0.004 - 23.7]);
        let mountains = smooth(-0.18, 0.48, region);
        let ridge = (1.0 - fractal(&self.ridge, wx * 0.022, wy * 0.022, 4).abs()).powi(3);
        let channel = self.ridge.get([wx * 0.035 - 19.1, wy * 0.035 + 71.9]).abs();
        let ravine = (1.0 - smooth(0.015, 0.13, channel)) * mountains;
        let plains = 1.0 - smooth(-0.45, 0.0, region);
        let mut height =
            continent * 7.0 + hills * (1.4 + mountains * 3.0) + ridge * mountains * 12.0
                - ravine * 5.5;
        height *= 1.0 - plains * 0.7;
        let plateau = smooth(
            0.25,
            0.65,
            self.detail.get([wx * 0.012 + 7.4, wy * 0.012 - 55.6]),
        );
        height += ((height / 2.0).round() * 2.0 - height) * plateau * 0.8;
        let landform = if ravine > 0.5 {
            Landform::Ravine
        } else if plateau > 0.6 {
            Landform::Plateau
        } else if mountains > 0.6 {
            Landform::Mountain
        } else if plains > 0.65 {
            Landform::Plain
        } else {
            Landform::Hills
        };
        // Medios niveles permiten escarpes; los gradientes conservan pendientes locales.
        (
            ((height * 2.0).round() * 0.5).clamp(-10.0, 18.0) as f32,
            landform,
        )
    }
}
