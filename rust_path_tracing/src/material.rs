use crate::math::Vec3;

pub struct Material {
    pub color: Vec3,
    pub emission: Vec3,
    pub refraction: f32,

    pub reflective: f32,
    pub emissive: f32,
    pub refractive: f32,
}
