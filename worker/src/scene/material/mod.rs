use std::marker::{Send, Sync};

use math::{Vec2, Vec3};

use crate::scene::scene_node::ReferenceReplacer;

pub mod base;
pub mod material_input;
pub mod pbr;

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

pub enum GetColorResult {
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3),
}

#[typetag::serde(tag = "type")]
pub trait MaterialUninit {
    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn Material>;
}

pub trait Material: Send + Sync {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult;
}
