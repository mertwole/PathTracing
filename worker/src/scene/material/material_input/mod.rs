use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

pub mod texture;

use texture::{Texture, TextureUninit};

use crate::scene::{
    scene_node::ReferenceReplacer, ResourceIdUninit, ResourceReferenceUninit, Scene,
};

pub type MaterialInputUninit = MaterialInputGeneric<TextureUninit>;
pub type MaterialInput = MaterialInputGeneric<Texture>;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialInputGeneric<T> {
    Color { color: Vec3 },
    Texture(T),
}

impl Default for MaterialInputUninit {
    fn default() -> MaterialInputUninit {
        MaterialInputUninit::Color {
            color: Vec3::new_xyz(1.0),
        }
    }
}

impl MaterialInputUninit {
    pub fn init(self, reference_replacer: &mut dyn ReferenceReplacer) -> MaterialInput {
        match self {
            MaterialInputGeneric::Color { color } => MaterialInput::Color { color },
            MaterialInputGeneric::Texture(texture) => {
                MaterialInput::Texture(texture.init(reference_replacer))
            }
        }
    }

    pub fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        match self {
            MaterialInputGeneric::Color { .. } => HashSet::new(),
            MaterialInputGeneric::Texture(texture) => texture.collect_references(),
        }
    }
}

impl MaterialInput {
    pub fn sample(&self, scene: Arc<Scene>, uv: Vec2) -> Vec3 {
        match self {
            MaterialInput::Color { color } => *color,
            MaterialInput::Texture(texture) => texture.sample(scene, uv),
        }
    }
}
