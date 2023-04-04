use std::marker::{Send, Sync};

use crate::{
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::scene_node::ReferenceReplacer,
};

pub mod base;
pub mod material_input;
pub mod pbr;

#[typetag::serde(tag = "type")]
pub trait MaterialUninit {
    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn Material>;
}

pub trait Material: Send + Sync + cpu_renderer::Material {}
