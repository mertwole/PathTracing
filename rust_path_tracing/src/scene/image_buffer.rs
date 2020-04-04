use math::Vec3;

pub struct ImageBuffer {
    pixels: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
}

impl ImageBuffer {
    pub fn new(width : usize, height : usize) -> ImageBuffer {
        ImageBuffer { width, height, pixels: vec![Vec3::zero(); width * height] }
    }

    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut Vec3 {
        &mut self.pixels[x + y * self.width]
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vec3{
        self.pixels[x + y * self.width].clone()
    }
}