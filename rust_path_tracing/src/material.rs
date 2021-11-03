use crate::math::{Vec3, Math};
use crate::rand::*;
use std::marker::{Send, Sync};
use crate::ray::RayTraceResult;

pub enum GetColorResult{
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3)
}

pub trait Material : Send + Sync {
    fn get_color(&self, dir : &Vec3, trace_result : &RayTraceResult) -> GetColorResult;
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

impl BaseMaterial {
    fn fresnel_reflection(theta_cos : f32, refraction : f32) -> f32 {
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

impl Material for BaseMaterial {
    fn get_color(&self, dir : &Vec3, trace_result : &RayTraceResult) -> GetColorResult{
        let random_num = thread_rng().gen_range(0.0, 1.0);
        if random_num < self.reflective {
            // reflect
            return GetColorResult
            ::NextRayColorMultiplierAndDirection(Vec3::new_xyz(1.0), dir.reflect(&trace_result.normal));
        } else if random_num < self.reflective + self.emissive {
            // emit
            return GetColorResult::Color(self.emission.clone());
        } else if random_num < self.reflective + self.emissive + self.refractive {
            // refract
            let cos = (&Vec3::zero() - &dir).dot(&trace_result.normal);
            let refraction = if trace_result.hit_inside { self.refraction } else { 1.0 / self.refraction };
            let fresnel = Self::fresnel_reflection(cos, refraction);

            //return GetColorResult::Color(Vec3::new_xyz(fresnel));

            let rand = thread_rng().gen_range(0.0, 1.0);
            let new_dir = if rand < fresnel {
                // reflect
                dir.reflect(&trace_result.normal)
            } else {
                // refract
                match dir.refract(&trace_result.normal, refraction) {
                    Some(direction) => { direction }
                    None => {
                        //return GetColorResult::Color(Vec3::new(100.0, 0.0, 0.0));
                        let reflected = dir.reflect(&trace_result.normal);
                        &reflected * if trace_result.hit_inside { -1.0 } else { 1.0 }
                    }
                }
            };

            return GetColorResult
            ::NextRayColorMultiplierAndDirection(Vec3::new_xyz(1.0), new_dir);
        } else {
            // diffuse
            let mut new_direction = Vec3::random_on_unit_sphere(thread_rng().gen_range(0.0, 1.0), thread_rng().gen_range(0.0, 1.0));

            if new_direction.dot(&trace_result.normal) < 0.0 {
                new_direction = &Vec3::zero() - &new_direction;
            }

            return GetColorResult
            ::NextRayColorMultiplierAndDirection(self.color.clone(), new_direction);
        };
    }
}

pub struct PBRMaterial{
    albedo : Vec3,
    roughness : f32,
    metallic : f32,
    // Precomputed
    f0 : Vec3,
    roughness_sqr : f32,
}

impl PBRMaterial{
    pub fn new(albedo : Vec3, roughness : f32, metallic : f32) -> PBRMaterial{
        PBRMaterial {
            f0 : &(&albedo * metallic) + &(&Vec3::new_xyz(0.04) * (1.0 - metallic)), 
            roughness_sqr : roughness * roughness,
            albedo,
            roughness,
            metallic,         
        }
    }

    fn ndf(&self, nh : f32) -> f32 {
        let nh = if nh > 0.95 { 0.95 } else { nh };
        let denom_sqrt = nh * nh * (self.roughness_sqr - 1.0) + 1.0;
        self.roughness_sqr / (denom_sqrt * denom_sqrt * math::PI)
    }

    fn geometry(&self, angle_cos : f32) -> f32 {
        let k = self.roughness_sqr * 0.5;

        //let mut k = 1.0 + self.roughness;
        //k = k * k * 0.125;

        angle_cos / (angle_cos * (1.0 - k) + k)
    }

    fn fresnel(&self, hi : f32) -> Vec3 {
        &self.f0 + &(&(&Vec3::new_xyz(1.0) - &self.f0) * (1.0 - hi).powi(5))
    }

    fn brdf(&self, normal : &Vec3, input_dir : &Vec3, output_dir : &Vec3) -> Vec3 {
        let ni = normal.dot(input_dir);
        let no = normal.dot(output_dir);
        let h = (output_dir + input_dir).normalized();

        let geometry = self.geometry(ni) * self.geometry(no);
        let ndf = self.ndf(normal.dot(&h));

        let specular_k = self.fresnel(h.dot(&input_dir));
        let diffuse_k = &(&Vec3::new_xyz(1.0) - &specular_k) * (1.0 - self.metallic);

        let diffuse = math::INV_PI * &self.albedo;
        let specular = geometry * ndf / (4.0 * ni * no);

        &(&specular_k * specular) + &(&diffuse_k * &diffuse)
    }
}

impl Material for PBRMaterial{
    fn get_color(&self, dir : &Vec3, trace_result : &RayTraceResult) -> GetColorResult{
        let rand = thread_rng().gen_range(0.0, 1.0);
        let reflect_prob = 1.0 - self.roughness;
        let reflect_dispersion = 0.001;
        let random_dir_prob = (1.0 - reflect_prob) / (1.0 - reflect_dispersion);
        let (selection_prob, output_dir) = if rand < reflect_prob {
            // Reflect
            let reflected = dir.reflect(&trace_result.normal);
            let rand_dir = Vec3::random_in_solid_angle(&reflected, 
                reflect_dispersion * 4.0 * math::PI,
                thread_rng().gen_range(0.0, 1.0),
                thread_rng().gen_range(0.0, 1.0));

            let prob = random_dir_prob + (1.0 - random_dir_prob) / reflect_dispersion;
            (prob, rand_dir)
        } else {
            // Random
            let mut dir = Vec3::random_on_unit_sphere(
                thread_rng().gen_range(0.0, 1.0), 
                thread_rng().gen_range(0.0, 1.0)
            );
            dir = if dir.dot(&trace_result.normal) < 0.0 { &dir * -1.0 } else { dir };

            (random_dir_prob, dir)
        };

        let input_dir = &Vec3::zero() - &dir;
        let mut mul = self.brdf(&trace_result.normal, &input_dir, &output_dir);

        mul = math::PI * output_dir.dot(&trace_result.normal) * &mul;
        mul = &mul / selection_prob;

        return GetColorResult
        ::NextRayColorMultiplierAndDirection(mul, output_dir);
    }
}