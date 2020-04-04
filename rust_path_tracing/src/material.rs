use crate::math::Vec3;

extern crate rand;
use rand::Rng;

pub enum GetColorResult{
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3)
}

pub trait Material {
    fn get_color(&self, dir : &Vec3, normal : &Vec3, rng : &mut rand::prelude::ThreadRng) -> GetColorResult;
}

pub struct BaseMaterial {
    pub color: Vec3,
    pub emission: Vec3,
    pub refraction: f32,

    pub reflective: f32,
    pub emissive: f32,
    pub refractive: f32,
}

impl BaseMaterial{
    pub fn default() -> BaseMaterial{
        BaseMaterial { 
            color : Vec3::new_xyz(1.0), 
            emission : Vec3::new_xyz(1.0), 
            refraction : 1.0, 
            reflective : 0.0, 
            emissive : 0.0, 
            refractive : 0.0 
        }
    }
}

pub struct PBRMaterial{

}

impl Material for BaseMaterial {
    fn get_color(&self, dir : &Vec3, normal : &Vec3, rng : &mut rand::prelude::ThreadRng) -> GetColorResult{
        let random_num = rng.gen_range(0.0, 1.0);
        if random_num < self.reflective {
            // reflect
            return GetColorResult
            ::NextRayColorMultiplierAndDirection(Vec3::new_xyz(1.0), dir.reflect(&normal));
        } else if random_num < self.reflective + self.emissive {
            // emit
            return GetColorResult::Color(self.emission.clone());
        } else if random_num < self.reflective + self.emissive + self.refractive {
            // refract
            return GetColorResult::Color(Vec3::zero());
        // TODO
        } else {
            // diffuse
            let mut new_direction = Vec3::new(
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
            ).normalized();
            if new_direction.dot(&normal) < 0.0 {
                new_direction = &Vec3::zero() - &new_direction;
            }

            return GetColorResult
            ::NextRayColorMultiplierAndDirection(self.color.clone(), new_direction);
        };
    }
}
