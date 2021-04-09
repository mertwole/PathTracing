use crate::math::Vec3;
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

impl Material for BaseMaterial {
    fn get_color(&self, dir : &Vec3, trace_result : &RayTraceResult) -> GetColorResult{
        //let mut rng = rand::prelude::thread_rng();
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
            let a = if trace_result.hit_inside { -1.0 } else { 1.0 };
            let refraction = if trace_result.hit_inside { self.refraction } else { 1.0 / self.refraction };
            let new_dir = match dir.refract(&trace_result.normal, refraction) {
                Some(direction) => { direction }
                None => { dir.cross(&(&trace_result.normal * a)).cross(&(&trace_result.normal * a)).normalized() }
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
    fresnel_k : Vec3,
    roughness_sqr : f32
}

impl PBRMaterial{
    pub fn new(albedo : Vec3, roughness : f32, metallic : f32) -> PBRMaterial{
        PBRMaterial {
            fresnel_k : &(&albedo * metallic) + &(&Vec3::new_xyz(0.04) * (1.0 - metallic)), 
            roughness_sqr : roughness * roughness,
            albedo,
            roughness,
            metallic,         
        }
    }

    fn fresnel(&self, nl : f32) -> Vec3{
        &self.fresnel_k + &(&(&Vec3::new_xyz(1.0) - &self.fresnel_k) * (1.0 - nl).powi(5))
    }

    fn ggx_microfacet_distribution(&self, nh : f32) -> f32{
        let nh_sqr = nh * nh;
        let denominator_sqrt = 1.0 - nh_sqr * (1.0 - self.roughness_sqr);
        self.roughness_sqr * math::INV_PI / (denominator_sqrt * denominator_sqrt)
    }

    fn ggx_selfshadowing(&self, angle_cos : f32) -> f32{
        let cos_sqr = angle_cos * angle_cos;
        2.0 / (1.0 + (1.0 + self.roughness_sqr * ((1.0 - cos_sqr) / cos_sqr)).sqrt())
    }
}

impl Material for PBRMaterial{
    fn get_color(&self, dir : &Vec3, trace_result : &RayTraceResult) -> GetColorResult{
        let mut rng = rand::prelude::thread_rng();
        let mut v = Vec3::new(
            rng.gen_range(-1.0, 1.0),
            rng.gen_range(-1.0, 1.0),
            rng.gen_range(-1.0, 1.0),
        ).normalized();
        if v.dot(&trace_result.normal) < 0.0 { v = &Vec3::zero() - &v; }
        
        let l = &Vec3::zero() - &dir;
        let nl = trace_result.normal.dot(&l);
        let nv = trace_result.normal.dot(&v);
        let h = (&v + &l).normalized();
        let specular = self.fresnel(nl);
        let mut color = Vec3::zero();
        // Specular
        color = &color + &(&specular * (
        self.ggx_microfacet_distribution(trace_result.normal.dot(&h)) * 
        self.ggx_selfshadowing(nl) *
        self.ggx_selfshadowing(nv) / (4.0 * nv)));
        // Diffuse
        color = &color + &(&(&Vec3::new_xyz(1.0) - &specular) * &(&self.albedo * ((1.0 - self.metallic) * nl * math::INV_PI)));
        
        color = &color * math::PI;

        return GetColorResult
        ::NextRayColorMultiplierAndDirection(color, v);
    }
}