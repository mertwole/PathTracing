use rand::*;
use serde::{Deserialize, Serialize};

use math::Vec3;

use super::{
    material_input::{MaterialInput, MaterialInputUninit},
    GetColorResult, Material, MaterialUninit,
};
use crate::raytraceable::RayTraceResult;

pub type BaseMaterial = BaseMaterialGeneric<MaterialInput>;
type BaseMaterialUninit = BaseMaterialGeneric<MaterialInputUninit>;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct BaseMaterialGeneric<I> {
    pub color: I,
    pub emission: Vec3,
    pub refraction: f32,

    pub reflective: f32,
    pub emissive: f32,
    pub refractive: f32,
}

impl Default for BaseMaterialUninit {
    fn default() -> BaseMaterialUninit {
        BaseMaterialUninit {
            color: MaterialInputUninit::default(),
            emission: Vec3::new_xyz(1.0),
            refraction: 1.0,
            reflective: 0.0,
            emissive: 0.0,
            refractive: 0.0,
        }
    }
}

impl BaseMaterial {
    fn fresnel_reflection(theta_cos: f32, refraction: f32) -> f32 {
        let refr_sqr = refraction * refraction;

        let c = theta_cos * refraction;
        let g = (1.0 + c * c - refr_sqr).sqrt();

        let a = (g - c) / (g + c);
        let b_nom = c * (g + c) - refr_sqr;
        let b_den = c * (g - c) + refr_sqr;
        let b = b_nom / b_den;

        0.5 * a * a * (1.0 + b * b)
    }
}

#[typetag::serde(name = "base")]
impl MaterialUninit for BaseMaterialUninit {
    fn init(self: Box<Self>) -> Box<dyn Material> {
        Box::new(BaseMaterial {
            color: self.color.init(),
            emission: self.emission,
            refraction: self.refraction,

            reflective: self.reflective,
            emissive: self.emissive,
            refractive: self.refractive,
        })
    }
}

impl Material for BaseMaterial {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult {
        let random_num = thread_rng().gen_range(0.0..1.0);
        if random_num < self.reflective {
            // reflect
            GetColorResult::NextRayColorMultiplierAndDirection(
                Vec3::new_xyz(1.0),
                dir.reflect(trace_result.normal),
            )
        } else if random_num < self.reflective + self.emissive {
            // emit
            GetColorResult::Color(self.emission)
        } else if random_num < self.reflective + self.emissive + self.refractive {
            // refract
            let cos = (Vec3::default() - dir).dot(trace_result.normal);
            let refraction = if trace_result.hit_inside {
                self.refraction
            } else {
                1.0 / self.refraction
            };
            let fresnel = Self::fresnel_reflection(cos, refraction);

            let rand = thread_rng().gen_range(0.0..1.0);
            let new_dir = if rand < fresnel {
                // reflect
                dir.reflect(trace_result.normal)
            } else {
                // refract
                match dir.refract(trace_result.normal, refraction) {
                    Some(direction) => direction,
                    None => {
                        let reflected = dir.reflect(trace_result.normal);
                        reflected * if trace_result.hit_inside { -1.0 } else { 1.0 }
                    }
                }
            };

            GetColorResult::NextRayColorMultiplierAndDirection(Vec3::new_xyz(1.0), new_dir)
        } else {
            // diffuse
            let mut new_direction = Vec3::random_on_unit_sphere(
                thread_rng().gen_range(0.0..1.0),
                thread_rng().gen_range(0.0..1.0),
            );

            if new_direction.dot(trace_result.normal) < 0.0 {
                new_direction = Vec3::default() - new_direction;
            }

            let color = self.color.sample(trace_result.uv);
            GetColorResult::NextRayColorMultiplierAndDirection(color, new_direction)
        }
    }
}
