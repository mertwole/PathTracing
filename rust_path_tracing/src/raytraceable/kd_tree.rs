use super::Raytraceable;
use crate::math::*;
use crate::ray::*;

pub struct KDTree{

}

impl KDTree{
    pub fn from_file() -> KDTree{
        KDTree { }
    }
}

impl Raytraceable for KDTree{
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult{
        let result = RayTraceResult::void();

        result
    }
}