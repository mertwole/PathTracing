use math::{HdrColor, Vec3};

pub struct ImageBuffer {
    pixels: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
}

impl ImageBuffer {
    pub fn new(width: usize, height: usize) -> ImageBuffer {
        ImageBuffer {
            width,
            height,
            pixels: vec![Vec3::default(); width * height],
        }
    }

    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut Vec3 {
        &mut self.pixels[x + y * self.width]
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vec3 {
        self.pixels[x + y * self.width]
    }

    pub fn get_pixel_vec(&self, color_multiplier: f32) -> Vec<Vec<HdrColor>> {
        let mut image_data: Vec<Vec<HdrColor>> = Vec::with_capacity(self.width);

        for img_x in 0..self.width {
            let mut image_column_data: Vec<HdrColor> = Vec::with_capacity(self.height);
            for img_y in 0..self.height {
                let mut pixel = self.get_pixel(img_x, img_y);
                pixel = pixel * color_multiplier;

                //let exposure = 0.5;
                // // Tonemapping
                // pixel.x = 1.0 - f32::exp(-pixel.x * exposure);
                // pixel.y = 1.0 - f32::exp(-pixel.y * exposure);
                // pixel.z = 1.0 - f32::exp(-pixel.z * exposure);
                // // Gamma correction
                // pixel.x = pixel.x.powf(1.0 / 2.2);
                // pixel.y = pixel.y.powf(1.0 / 2.2);
                // pixel.z = pixel.z.powf(1.0 / 2.2);

                image_column_data.push(HdrColor::from_vec3(pixel));
            }
            image_data.push(image_column_data);
        }

        image_data
    }
}
