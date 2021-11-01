use math::*;

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

    pub fn get_pixel_vec(&self, color_multiplier : f32) -> Vec<Vec<Color24bpprgb>>{
        let mut image_data : Vec<Vec<Color24bpprgb>> = Vec::with_capacity(self.width);
       
        for img_x in 0..self.width{
            let mut image_column_data : Vec<Color24bpprgb> = Vec::with_capacity(self.height);
            for img_y in 0..self.height{
                let mut pixel = self.get_pixel(img_x, img_y);
                pixel = &pixel * color_multiplier;
                // Tonemapping
                //pixel = &pixel / &(&pixel + &Vec3::new_xyz(1.0));
                // Gamma correction
                pixel.x = pixel.x.powf(1.0 / 2.2);
                pixel.y = pixel.y.powf(1.0 / 2.2);
                pixel.z = pixel.z.powf(1.0 / 2.2);

                let (r, g, b) = (pixel.x, pixel.y, pixel.z);
                image_column_data.push(Color24bpprgb::from_normalized(r, g, b));
            }
            image_data.push(image_column_data);
        }

        image_data
    }
}