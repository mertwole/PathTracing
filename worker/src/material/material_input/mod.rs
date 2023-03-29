use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

pub mod texture;

use texture::{Texture, TextureUninit};

use crate::scene::scene_node::ReferenceReplacer;

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
}

impl MaterialInput {
    pub fn sample(&self, uv: Vec2) -> Vec3 {
        match self {
            MaterialInput::Color { color } => *color,
            MaterialInput::Texture(texture) => texture.sample(uv),
        }
    }
}
