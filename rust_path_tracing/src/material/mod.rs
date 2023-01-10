use std::marker::{Send, Sync};

use math::Vec3;

use crate::raytraceable::RayTraceResult;

pub mod base;
pub mod pbr;

pub enum GetColorResult {
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3),
}

#[typetag::serde(tag = "type")]
pub trait MaterialUninit {
    fn init(self: Box<Self>) -> Box<dyn Material>;
}

pub trait Material: Send + Sync {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult;
}
