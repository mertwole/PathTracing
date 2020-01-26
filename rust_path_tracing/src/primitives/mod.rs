use crate::ray::{Ray, RayTraceResult};
pub mod sphere;
pub use sphere::*;
pub mod plane;
pub use plane::*;

pub trait Raytraceable{
    fn TraceRay(&self, ray : &Ray) -> RayTraceResult;
}