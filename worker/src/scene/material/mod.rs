use crate::renderer::cpu_renderer::{self, RayTraceResult};

use super::Initializable;

pub mod base;
pub mod material_input;
pub mod pbr;

#[typetag::serde(tag = "type")]
pub trait MaterialUninit: Initializable<Initialized = Box<dyn Material>> {}

pub trait Material: cpu_renderer::Material {}
