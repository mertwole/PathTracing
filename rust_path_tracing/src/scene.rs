use crate::camera::*;
use crate::material::*;
use crate::math::*;
use crate::raytraceable::*;
use crate::ray::{Ray, RayTraceResult};

extern crate image;
extern crate rand;
use rand::Rng;

struct ImageBuffer {
    pixels: Vec<Vec3>,
    width: usize,
    height: usize,
}

impl ImageBuffer {
    fn new(width : usize, height : usize) -> ImageBuffer {
        let mut img_buf = ImageBuffer { width, height, pixels: Vec::with_capacity(width * height) };
        for _i in 0..width * height { 
            img_buf.pixels.push(Vec3::zero()); 
        }
        img_buf
    }

    fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut Vec3 {
        &mut self.pixels[x + y * self.width]
    }

    fn save_to_file(&self, path: &std::path::Path) {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);
        for pixel in &self.pixels {
            buffer.push((pixel.x.powf(1.0 / 2.2) * 255f32) as u8);
            buffer.push((pixel.y.powf(1.0 / 2.2) * 255f32) as u8);
            buffer.push((pixel.z.powf(1.0 / 2.2) * 255f32) as u8);
        }

        image::save_buffer_with_format(
            path,
            &buffer,
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Bmp,
        ).unwrap();
    }
}

pub struct Scene {
    camera: Camera,

    image: ImageBuffer,
    iteration: u32,

    primitives: Vec<Box<dyn Raytraceable>>,
    materials: Vec<Material>,

    rng: rand::prelude::ThreadRng,
}

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            image: ImageBuffer::new(camera.width, camera.height),
            camera,
            iteration: 0u32,
            primitives: Vec::new(),
            materials: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn add_primitive(&mut self, primitive: Box<dyn Raytraceable>) {
        self.primitives.push(primitive);
    }

    pub fn init_materials(&mut self, materials: Vec<Material>) {
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

    fn get_color(&mut self, ray: &Ray, max_depth: u32) -> Vec3 {
        if max_depth == 0 {
            return Vec3::zero();
        }

        let trace_result = self.trace_ray(ray);

        if !trace_result.hit {
            return Vec3::zero();
        }

        let random_num = self.rng.gen_range(0.0, 1.0);
        let material = &self.materials[trace_result.material_id];

        let color = if random_num < material.reflective {
            // reflect
            let new_ray = Ray::new(
                trace_result.point,
                ray.direction.reflect(&trace_result.normal),
                math::EPSILON,
                std::f32::MAX,
            );
            self.get_color(&new_ray, max_depth - 1)
        } else if random_num < material.reflective + material.emissive {
            // emit
            material.emission
        } else if random_num < material.reflective + material.emissive + material.refractive {
            // refract
            Vec3::zero()
        // TODO
        } else {
            // diffuse
            let mut new_direction = Vec3::new(
                self.rng.gen_range(-1.0, 1.0),
                self.rng.gen_range(-1.0, 1.0),
                self.rng.gen_range(-1.0, 1.0),
            )
            .normalized();
            if new_direction.dot(&trace_result.normal) < 0.0 {
                new_direction = &new_direction * -1.0;
            }

            let new_ray = Ray::new(
                trace_result.point,
                new_direction,
                math::EPSILON,
                std::f32::MAX,
            );
            let new_color = &material.color.clone();
            new_color * &self.get_color(&new_ray, max_depth - 1)
        };

        color
    }

    pub fn iteration(&mut self) {
        let inv_iter = 1.0 / (self.iteration as f32 + 1.0);
        for x in 0..self.camera.width {
            for y in 0..self.camera.height {
                let color = self.get_color(&self.camera.get_ray(x, y), 16);
                let pixel = self.image.get_pixel_mut(x, y);
                let new_color = &(&*pixel * self.iteration as f32) + &color;
                *pixel = &new_color * inv_iter;
            }

            println!("iteration : {} x : {}", self.iteration, x);
        }

        self.iteration += 1;
    }

    pub fn save_output(&self, path: &std::path::Path) {
        self.image.save_to_file(path);
    }
}
