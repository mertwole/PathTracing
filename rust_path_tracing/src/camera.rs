use crate::math::*;
use crate::ray::*;

pub struct Camera {
    pub resolution : UVec2,
    pub viewport: Vec2,
    pub rotation: Mat3,
    pub view_distance: f32,
    pub position: Vec3,
}

impl Camera {
    pub fn get_ray(&self, point : UVec2) -> Ray {
        let mut watch_dot = self.position.clone();
        watch_dot.z -= self.view_distance;
        watch_dot.x += ((point.x as f32) / (self.resolution.x as f32) - 0.5) * self.viewport.x;
        watch_dot.y += ((point.y as f32) / (self.resolution.y as f32) - 0.5) * self.viewport.y;
        let direction = &self.rotation * &(&watch_dot - &self.position);

        Ray::new(
            self.position,
            direction.normalized(),
            direction.length(),
            std::f32::MAX,
        )
    }
}
