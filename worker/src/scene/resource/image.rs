use std::collections::HashSet;

use image::RgbaImage;
use math::{Vec2, Vec3};

use super::{ReferenceReplacer, Resource, ResourceReferenceUninit};

pub struct Image(RgbaImage);

impl Resource for Image {
    type Initialized = Image;

    fn load(data: &[u8]) -> Self {
        Image(
            image::load_from_memory(&data)
                .expect("Incorrect image format")
                .to_rgba8(),
        )
    }

    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        HashSet::new()
    }

    fn init(self, _: &mut dyn ReferenceReplacer) -> Self::Initialized {
        self
    }
}

impl Image {
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    pub fn height(&self) -> u32 {
        self.0.height()
    }

    pub fn get_pixel(&self, coords: Vec2) -> Vec3 {
        let pixel = self.0.get_pixel(coords.x as u32, coords.y as u32);
        Vec3::new(
            pixel.0[0] as f32 / 256.0,
            pixel.0[1] as f32 / 256.0,
            pixel.0[2] as f32 / 256.0,
        )
    }
}
