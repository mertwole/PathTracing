use crate::primitives::*;
use crate::camera::*;
use crate::ray::{RayTraceResult, Ray};
use crate::math::*;
use crate::camera::{IMAGE_WIDTH, IMAGE_HEIGHT};

pub struct Scene{
    pub camera : Camera,

    pub image : [[Vec3; IMAGE_WIDTH]; IMAGE_HEIGHT],
    iteration : u32,

    pub spheres : Vec<Sphere>,
    pub planes : Vec<Plane>
}

impl Scene{
    pub fn new(camera : Camera) -> Scene{
        Scene{camera, image : [[Vec3::zero(); IMAGE_WIDTH]; IMAGE_HEIGHT], iteration : 0u32, spheres : Vec::new(), planes : Vec::new()}
    }

    fn TraceRay(&self, ray : &Ray) -> RayTraceResult{
        let mut result = RayTraceResult::void();
        result.t = 100000000f32; 
        
        for sphere in &self.spheres {
            let sphere_result = sphere.TraceRay(&ray);
            if result.hit && sphere_result.t < result.t{
                result = sphere_result;
            }    
        }

        for plane in &self.planes{
            let plane_result = plane.TraceRay(&ray);
            if result.hit && plane_result.t < result.t{
                result = plane_result;
            }
        }

        result
    }

    fn GetColor(&self, ray : Ray) -> Vec3{
        let color = Vec3::zero();

        color
    }

    pub fn Iteration(&mut self){
        self.iteration += 1;

        for x in 0..IMAGE_WIDTH - 1{
            for y in 0..IMAGE_HEIGHT - 1{
                let color = self.GetColor(self.camera.get_ray_by_norm_screen_coords(Vec2::new(0.0, 0.0)));
                let old_color = self.image[y][x];
                let mut new_color = &(&old_color * self.iteration as f32) + &color;
                new_color = &new_color / (self.iteration as f32 + 1f32);
                self.image[y][x] = new_color;
            }
        }
    }
}