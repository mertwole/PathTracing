use crate::raytraceable::RayTraceResult;
use math::Vec3;
use std::marker::{Send, Sync};

pub mod base;
pub mod pbr;

pub enum GetColorResult {
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3),
}

pub trait Material: Send + Sync {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult;
}
