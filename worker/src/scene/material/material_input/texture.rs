use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use crate::scene::scene_node::{
    ReferenceReplacer, ResourceId, ResourceIdUninit, ResourceReferenceUninit, ResourceType,
};

pub type TextureUninit = TextureGeneric<ResourceIdUninit>;
pub type Texture = TextureGeneric<ResourceId>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UvMode {
    Clamp,
    Repeat,
}

// @TODO: Add support for different texture formats
#[derive(Serialize, Deserialize)]
pub struct TextureGeneric<I> {
    #[serde(rename = "path")]
    image: I,
    uv_mode: UvMode,
}

impl TextureUninit {
    pub fn init(self, reference_replacer: &mut dyn ReferenceReplacer) -> Texture {
        Texture {
            image: reference_replacer
                .get_replacement(ResourceReferenceUninit {
                    path: self.image,
                    ty: ResourceType::Image,
                })
                .path,
            uv_mode: self.uv_mode,
        }
    }
}

impl Texture {
    pub fn sample(&self, uv: Vec2) -> Vec3 {
        // let mut uv = match self.uv_mode {
        //     UvMode::Clamp => Vec2::new(uv.x.clamp(0.0, 1.0), uv.y.clamp(0.0, 1.0)),
        //     UvMode::Repeat => Vec2::new(uv.x - uv.x.floor(), uv.y - uv.y.floor()),
        // };

        // uv.y = 1.0 - uv.y;

        // let coords = uv
        //     * Vec2::new(
        //         (self.image.width() - 1) as f32,
        //         (self.image.height() - 1) as f32,
        //     );
        // let pixel = self.image.get_pixel(coords.x as u32, coords.y as u32);
        // Vec3::new(
        //     pixel.0[0] as f32 / 256.0,
        //     pixel.0[1] as f32 / 256.0,
        //     pixel.0[2] as f32 / 256.0,
        // )
        unimplemented!()
    }
}
