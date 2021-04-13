use crate::math::*;
use crate::ray::*;
use crate::rand::*;

pub enum BokehShape {
    Point,
    Circle,
    Square
}

impl BokehShape {
    fn sample(&self) -> Vec2 {
        match self {
            BokehShape::Point => { 
                Vec2::new(0.0, 0.0) 
            }
            BokehShape::Circle => { 
                let mut rng = rand::prelude::thread_rng();
                let phi = rng.gen_range(0.0, 2.0 * math::PI);
                let r = f32::sqrt(rng.gen_range(0.0, 1.0));
                let phi_sin_cos = f32::sin_cos(phi);
                r * &Vec2::new(phi_sin_cos.1, phi_sin_cos.0)
            }
            BokehShape::Square => { 
                let mut rng = rand::prelude::thread_rng();
                Vec2::new(rng.gen_range(-0.5, 0.5), rng.gen_range(-0.5, 0.5)) 
            }
        }
    }
}

pub struct Camera {
    pub resolution : UVec2,
    pub rotation: Mat3,
    pub position: Vec3,

    pub fov : f32,
    pub near_plane : f32,
    pub focal_length : f32,

    pub bokeh_shape : BokehShape,
    pub bokeh_size : f32
}

impl Camera {
    pub fn get_ray(&self, point : UVec2) -> Ray {
        let mut rng = rand::prelude::thread_rng();
        let x_offset = rng.gen_range(-0.5, 0.5);
        let y_offset = rng.gen_range(-0.5, 0.5);

        let mut viewport = Vec2::new(self.focal_length * f32::tan(self.fov * 0.5) * 2.0, 0.0);
        viewport.y = viewport.x * (self.resolution.y as f32 / self.resolution.x as f32);

        let mut watch_dot = self.position.clone();
        watch_dot.x += ((point.x as f32 + x_offset) / (self.resolution.x as f32) - 0.5) * viewport.x;
        watch_dot.y += ((point.y as f32 + y_offset) / (self.resolution.y as f32) - 0.5) * viewport.y;

        watch_dot.z -= self.focal_length;

        let mut point_on_objective = self.position.clone();

        let objective_sample = &self.bokeh_shape.sample() * self.bokeh_size;
        point_on_objective.x += objective_sample.x;
        point_on_objective.y += objective_sample.y;
        //point_on_objective.z += self.focal_length;

        let direction = &self.rotation * &(&watch_dot - &point_on_objective);

        let near_plane_dist = direction.length() / self.focal_length * self.near_plane; 

        Ray::new(
            point_on_objective,
            direction.normalized(),
            near_plane_dist,
            std::f32::MAX,
        )
    }
}
