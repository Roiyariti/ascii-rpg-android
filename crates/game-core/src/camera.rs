use glam::{Vec2, Vec3};

/// Cámara ortográfica. x/y son el suelo; z es altura. Independiente de los turnos.
#[derive(Debug, Clone, Copy)]
pub struct IsoCamera {
    pub target: Vec3,
    pub viewport: Vec2,
    pub tile_half: Vec2,
    pub height_scale: f32,
    pub zoom: f32,
    pub pan: Vec2,
}
impl IsoCamera {
    pub fn new(viewport: Vec2) -> Self {
        Self {
            target: Vec3::ZERO,
            viewport,
            tile_half: Vec2::new(28.0, 14.0),
            height_scale: 22.0,
            zoom: 1.25,
            pan: Vec2::ZERO,
        }
    }
    /// Menor profundidad queda delante; coincide con el rayo de esta proyección.
    pub fn project(&self, world: Vec3) -> Vec3 {
        let delta = self.project_vector(world - self.target);
        Vec3::new(
            self.viewport.x * 0.5 + self.pan.x + delta.x,
            self.viewport.y * 0.48 + self.pan.y + delta.y,
            0.5 + delta.z,
        )
    }
    pub fn project_vector(&self, vector: Vec3) -> Vec3 {
        let view_z = 2.0 * self.tile_half.y / self.height_scale;
        Vec3::new(
            (vector.x - vector.y) * self.tile_half.x * self.zoom,
            ((vector.x + vector.y) * self.tile_half.y - vector.z * self.height_scale) * self.zoom,
            -(vector.x + vector.y + vector.z * view_z) / 256.0,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_centers_tracks_zoom_and_preserves_depth_along_view_ray() {
        let mut camera = IsoCamera::new(Vec2::new(1200.0, 600.0));
        camera.target = Vec3::new(5.0, -3.0, 7.0);
        assert_eq!(
            camera.project(camera.target).truncate(),
            Vec2::new(600.0, 288.0)
        );
        let far = camera.project(camera.target);
        let near = camera.project(camera.target + Vec3::new(1.0, 1.0, 28.0 / 22.0));
        assert!((near.truncate() - far.truncate()).length() < 0.001);
        assert!(near.z < far.z);
        let before = camera.project_vector(Vec3::X);
        camera.zoom *= 2.0;
        assert_eq!(
            camera.project_vector(Vec3::X).truncate(),
            before.truncate() * 2.0
        );
        camera.pan = Vec2::new(12.0, -8.0);
        assert_eq!(
            camera.project(camera.target).truncate(),
            Vec2::new(612.0, 280.0)
        );
    }
}
