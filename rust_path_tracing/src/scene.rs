use crate::camera::*;
use crate::math::*;
use crate::primitives::*;
use crate::ray::{Ray, RayTraceResult};
use crate::material::*;

extern crate image;

struct ImageBuffer{
    pixels : Vec<Vec3>,
    width : usize,
    height : usize
}

impl ImageBuffer{
    fn new(w : usize, h : usize) -> ImageBuffer{
        let mut img_buf = ImageBuffer{ width : w, height : h, pixels : Vec::with_capacity(w * h) };
        for _i in 0..w*h{
            img_buf.pixels.push(Vec3::zero());
        }
        img_buf
    }

    fn get_pixel_mut(&mut self, x : usize, y : usize) -> &mut Vec3{
        &mut self.pixels[x + y * self.width]
    }

    fn save_to_file(&self, path : &std::path::Path) {
        let mut buffer : Vec<u8> = Vec::with_capacity(self.width * self.height * 3);
        for pixel in &self.pixels{
            buffer.push((pixel.x * 255f32) as u8);
            buffer.push((pixel.y * 255f32) as u8);
            buffer.push((pixel.z * 255f32) as u8);
        }

        image::save_buffer_with_format(path, &buffer, self.width as u32, self.height as u32, image::ColorType::Rgb8 , image::ImageFormat::Bmp).unwrap();
    }
}

pub struct Scene {
    camera: Camera,

    image: ImageBuffer,
    iteration: u32,

    primitives: Vec<Box<dyn Raytraceable>>,
    materials : Vec<Material>
}

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            image: ImageBuffer::new(camera.width, camera.height),
            camera,         
            iteration: 0u32,
            primitives: Vec::new(),
            materials : Vec::new()
        }
    }

    pub fn add_primitive(&mut self, primitive: Box<dyn Raytraceable>) {
        self.primitives.push(primitive);
    }

    pub fn init_materials(&mut self, materials : Vec<Material>) {
        self.materials = materials;
    }

    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        result.t = std::f32::MAX;

        for primitive in &self.primitives {
            let primitive_result = primitive.trace_ray(&ray);
            if primitive_result.hit && primitive_result.t < result.t {
                result = primitive_result;
            }
        }

        result
    }

    fn get_color(&self, ray: &Ray) -> Vec3 {
        let color = if(self.trace_ray(ray).hit) { self.materials[self.trace_ray(ray).material_id].color } else { Vec3::zero() };

        // TODO : actually compute color

        color
    }

    pub fn iteration(&mut self) {
        self.iteration += 1;

        for x in 0..self.camera.width {
            for y in 0..self.camera.height {
                let color = self.get_color(&self.camera.get_ray(x, y));
                let pixel = self.image.get_pixel_mut(x, y);
                let mut new_color = &(&*pixel * self.iteration as f32) + &color;
                new_color = &new_color / (self.iteration as f32 + 1f32);
                *pixel = new_color;
            }
        }
    }

    pub fn save_output(&self, path: &std::path::Path) {
        self.image.save_to_file(path);
    }
}
