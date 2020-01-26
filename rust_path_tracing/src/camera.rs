use crate::math::*;
use crate::ray::*;

pub const IMAGE_WIDTH : usize = 1920;
pub const IMAGE_HEIGHT : usize = 1080;

pub struct Camera{

}

impl Camera{
    pub fn get_ray_by_norm_screen_coords(&self, coords : Vec2) -> Ray{
        let ray = Ray::new(Vec3::zero(), Vec3::zero(), 0.0, 0.0);

        ray
    }
}