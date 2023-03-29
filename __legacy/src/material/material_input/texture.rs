use image::{io::Reader as ImageReader, RgbaImage};
use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

pub type TextureUninit = TextureGeneric<String>;
pub type Texture = TextureGeneric<RgbaImage>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UvMode {
    Clamp,
    Repeat,
}

// @TODO: Add sipport for different texture formats
#[derive(Serialize, Deserialize)]
pub struct TextureGeneric<I> {
    #[serde(rename = "path")]
    image: I,
    uv_mode: UvMode,
}

impl TextureUninit {
    pub fn init(self) -> Texture {
        let image_init = ImageReader::open(&self.image)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        Texture {
            image: image_init,
            uv_mode: self.uv_mode,
        }
    }
}

impl Texture {
    pub fn sample(&self, uv: Vec2) -> Vec3 {
        let mut uv = match self.uv_mode {
            UvMode::Clamp => Vec2::new(uv.x.clamp(0.0, 1.0), uv.y.clamp(0.0, 1.0)),
            UvMode::Repeat => Vec2::new(uv.x - uv.x.floor(), uv.y - uv.y.floor()),
        };

        uv.y = 1.0 - uv.y;

        let coords = uv
            * Vec2::new(
                (self.image.width() - 1) as f32,
                (self.image.height() - 1) as f32,
            );
        let pixel = self.image.get_pixel(coords.x as u32, coords.y as u32);
        Vec3::new(
            pixel.0[0] as f32 / 256.0,
            pixel.0[1] as f32 / 256.0,
            pixel.0[2] as f32 / 256.0,
        )
    }
}
