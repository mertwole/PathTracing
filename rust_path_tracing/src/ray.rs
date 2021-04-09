use crate::math::*;

pub struct RayTraceResult {
    pub hit: bool,
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material_id: usize,
    pub hit_inside: bool
}

impl RayTraceResult {
    pub fn void() -> RayTraceResult {
        RayTraceResult {
            hit: false,
            point: Vec3::zero(),
            normal: Vec3::zero(),
            t: 0.0,
            material_id: 0,
            hit_inside : false
        }
    }
}

pub struct Ray {
    pub source: Vec3,
    pub direction: Vec3,

    pub min: f32,
    pub max: f32,
}

impl Ray {
    pub fn new(source: Vec3, direction: Vec3, min: f32, max: f32) -> Ray {
        Ray { source, direction, min, max }
    }
}
