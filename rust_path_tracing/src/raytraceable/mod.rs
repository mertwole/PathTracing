use crate::ray::Ray;
use math::Vec3;
use std::marker::{Send, Sync};

pub mod sphere;
pub use sphere::*;
pub mod plane;
pub use plane::*;
pub mod triangle;
pub use triangle::*;
pub mod kd_tree;
pub use self::kd_tree::*;

pub struct RayTraceResult {
    pub hit: bool,
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material_id: usize,
    pub hit_inside: bool,
}

impl RayTraceResult {
    pub fn void() -> RayTraceResult {
        RayTraceResult {
            hit: false,
            point: Vec3::default(),
            normal: Vec3::default(),
            t: 0.0,
            material_id: 0,
            hit_inside: false,
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait RaytraceableUninit {
    fn init(self: Box<Self>) -> Box<dyn Raytraceable>;
}

pub trait Raytraceable: Send + Sync {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult;
}
