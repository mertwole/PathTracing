use std::collections::HashSet;

pub mod base;
pub mod material_input;
pub mod pbr;

use crate::{
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::resource::{ReferenceReplacer, Resource, ResourceReferenceUninit},
};

#[typetag::serde(tag = "type")]
pub trait MaterialUninit {
    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn Material>;
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit>;
}

pub trait Material: cpu_renderer::Material {}

pub struct BoxedMaterial(Box<dyn MaterialUninit>);

impl Resource for BoxedMaterial {
    type Initialized = Box<dyn Material>;

    fn load(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let data = String::from_utf8(data.to_vec()).unwrap();
        BoxedMaterial(serde_json::de::from_str(&data).unwrap())
    }

    fn init(self, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn Material> {
        self.0.init(reference_replacer)
    }

    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        self.0.collect_references()
    }
}
