use std::sync::Arc;

use super::image_buffer::ImageBuffer;
use super::GetColorResult;
use crate::api::render_task::RenderTask;
use crate::ray::Ray;
use crate::scene::Scene;
use math::{HdrColor, UVec2, Vec3};

pub struct WorkGroup {
    iteration: usize,
    x_offset: usize,
    y_offset: usize,

    pub buffer: ImageBuffer,
}

impl WorkGroup {
    pub fn new(x_offset: usize, y_offset: usize, width: usize, height: usize) -> WorkGroup {
        WorkGroup {
            iteration: 0,
            x_offset,
            y_offset,
            buffer: ImageBuffer::new(width, height),
        }
    }

    fn get_color(&mut self, scene_data: Arc<Scene>, ray: &Ray, max_depth: usize) -> Vec3 {
        if max_depth == 0 {
            return Vec3::default();
        }

        let trace_result = scene_data.hierarchy.trace_ray(scene_data.clone(), ray);
        if !trace_result.hit {
            return Vec3::default();
        }

        let material = scene_data.materials[trace_result.material_id].as_ref();
        let color_result = material.get_color(ray.direction, &trace_result, scene_data.clone());

        match color_result {
            GetColorResult::Color(color) => color,
            GetColorResult::NextRayColorMultiplierAndDirection(mul, dir) => {
                let ray_start = trace_result.point + dir * math::EPSILON;
                let new_ray = Ray::new(ray_start, dir, math::EPSILON, std::f32::MAX);
                mul * self.get_color(scene_data, &new_ray, max_depth - 1)
            }
        }
    }

    pub fn iteration(&mut self, scene_data: Arc<Scene>, render_task: Arc<RenderTask>) {
        for x in 0..self.buffer.width {
            for y in 0..self.buffer.height {
                let ray = render_task
                    .camera
                    .get_ray(UVec2::new(self.x_offset + x, self.y_offset + y));
                let color =
                    self.get_color(scene_data.clone(), &ray, render_task.config.trace_depth);
                let pixel = self.buffer.get_pixel_mut(x, y);
                *pixel = *pixel + color;
            }
        }
        self.iteration += 1;
    }

    pub fn get_raw_image_data(&self) -> Vec<Vec<HdrColor>> {
        self.buffer.get_pixel_vec(1.0 / (self.iteration as f32))
    }
}
