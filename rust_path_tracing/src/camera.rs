use crate::math::*;
use crate::ray::*;

pub struct Camera {
    pub width: usize,
    pub height: usize,
}

impl Camera {
    pub fn new(width: usize, height: usize) -> Camera {
        Camera { width, height }
    }

    pub fn get_ray(&self, x: usize, y: usize) -> Ray {
        let ray = Ray::new(Vec3::zero(), Vec3::zero(), 0.0, 0.0);

        // TODO : actually compute ray

        ray
    }
}
