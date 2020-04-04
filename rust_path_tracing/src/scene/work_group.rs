use super::Scene;
use super::image_buffer::*;
use crate::ray::{Ray, RayTraceResult};
use math::Vec3;
use crate::material::*;

pub struct WorkGroup{
    iteration : usize,
    x_offset : usize,
    y_offset : usize,

    pub buffer : ImageBuffer,

    rng : rand::prelude::ThreadRng,
}

impl WorkGroup{
    pub fn new(x_offset : usize, y_offset : usize, width : usize, height : usize) -> WorkGroup{
        WorkGroup {
            iteration : 0,
            x_offset,
            y_offset,
            buffer : ImageBuffer::new(width, height),
            rng : rand::prelude::thread_rng()
        }
    }

    fn trace_ray(&self, scene : &Scene, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        result.t = std::f32::MAX;

        for primitive in &scene.primitives {
            let primitive_result = primitive.trace_ray(&ray);
            if primitive_result.hit && primitive_result.t < result.t {
                result = primitive_result;
            }
        }

        result
    }

    fn get_color(&mut self, scene : &Scene, ray: &Ray, max_depth: usize) -> Vec3 {
        if max_depth == 0 { return Vec3::zero(); }

        let trace_result = self.trace_ray(scene, ray);
        if !trace_result.hit { return Vec3::zero(); }

        let material = scene.materials[trace_result.material_id].as_ref();
        let color_result = material.get_color(&ray.direction, &trace_result.normal, &mut self.rng);

        match color_result{
            GetColorResult::Color(color) => { return color; }
            GetColorResult::NextRayColorMultiplierAndDirection(mul, dir) => { 
                let new_ray = Ray::new(trace_result.point, dir, math::EPSILON, std::f32::MAX);
                return &mul * &self.get_color(scene, &new_ray, max_depth - 1) 
            }
        };
    }

    pub fn iteration(&mut self, scene : &Scene, trace_depth : usize) {
        for x in 0..self.buffer.width {
            for y in 0..self.buffer.height {
                let color = self.get_color(scene, &scene.camera.get_ray(self.x_offset + x, self.y_offset + y), trace_depth);
                let pixel = self.buffer.get_pixel_mut(x, y);
                *pixel = &*pixel + &color;
            }
        }
        self.iteration += 1;
    }
}