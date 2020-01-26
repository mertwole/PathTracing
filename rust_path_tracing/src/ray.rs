use crate::math::*;

pub struct RayTraceResult{
    pub hit : bool,
    pub point : Vec3,
    pub normal : Vec3,
    pub t : f32
}

impl RayTraceResult{
    pub fn void() -> RayTraceResult{
        RayTraceResult{hit : false, point : Vec3::zero(), normal : Vec3::zero(), t : 0.0}
    }
}

pub struct Ray{
    pub source : Vec3,
    pub direction : Vec3,

    pub min : f32,
    pub max : f32
}