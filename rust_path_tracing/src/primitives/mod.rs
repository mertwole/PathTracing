use crate::ray::{Ray, RayTraceResult};

pub mod sphere;
pub use sphere::*;
pub mod plane;
pub use plane::*;
pub mod triangle;
pub use triangle::*;

pub trait Raytraceable {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult;
}
