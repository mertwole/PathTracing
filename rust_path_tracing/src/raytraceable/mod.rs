use crate::ray::Ray;
use math::{Vec2, Vec3};
use std::marker::{Send, Sync};

pub mod plane;
pub mod sphere;
pub mod triangle;

pub struct RayTraceResult {
    pub hit: bool,
    pub hit_inside: bool,

    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub t: f32,

    pub material_id: usize,
}

impl RayTraceResult {
    pub fn void() -> RayTraceResult {
        RayTraceResult {
            hit: false,
            point: Vec3::default(),
            normal: Vec3::default(),
            uv: Vec2::default(),
            t: 0.0,
            material_id: 0,
            hit_inside: false,
        }
    }
}

#[derive(Default)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    pub fn trace_ray(&self, ray: &Ray) -> bool {
        let inv_dir = Vec3::new_xyz(1.0) / ray.direction;
        let t0 = (self.min - ray.source) * inv_dir;
        let t1 = (self.max - ray.source) * inv_dir;

        let max_triple = |a: f32, b: f32, c: f32| a.max(b.max(c));
        let min_triple = |a: f32, b: f32, c: f32| a.min(b.min(c));

        let tmin = max_triple(t0.x.min(t1.x), t0.y.min(t1.y), t0.z.min(t1.z));
        let tmax = min_triple(t0.x.max(t1.x), t0.y.max(t1.y), t0.z.max(t1.z));
        tmin <= tmax
    }

    pub fn split(&self, plane_pos: Vec3, plane_normal: Vec3) -> (AABB, AABB) {
        let split_pos_along_normal = plane_pos * plane_normal;
        let one_minus_normal = Vec3::new(1.0, 1.0, 1.0) - plane_normal;
        (
            AABB::new(
                self.min,
                split_pos_along_normal + one_minus_normal * self.max,
            ),
            AABB::new(
                split_pos_along_normal + one_minus_normal * self.min,
                self.max,
            ),
        )
    }

    pub fn unite(&self, other: &AABB) -> AABB {
        let min = Vec3::new(
            f32::min(self.min.x, other.min.x),
            f32::min(self.min.y, other.min.y),
            f32::min(self.min.z, other.min.z),
        );

        let max = Vec3::new(
            f32::max(self.max.x, other.max.x),
            f32::max(self.max.y, other.max.y),
            f32::max(self.max.z, other.max.z),
        );

        AABB::new(min, max)
    }
}

#[typetag::serde(tag = "type")]
pub trait RaytraceableUninit {
    fn init(self: Box<Self>) -> Box<dyn Raytraceable>;
}

pub trait Bounded: Raytraceable {
    fn get_bounds(&self) -> AABB;
    // May be false-positive.
    fn intersect_with_aabb(&self, aabb: &AABB) -> bool;
}

// @TODO: Refactor 'Bounded' trait
pub trait Raytraceable: Send + Sync {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult;
    fn is_bounded(&self) -> bool;
    fn get_bounded(self: Box<Self>) -> Option<Box<dyn Bounded>>;
}
