use crate::ray::{Ray, RayTraceResult};

pub mod sphere;
pub use sphere::*;
pub mod plane;
pub use plane::*;
pub mod triangle;
pub use triangle::*;
pub mod kd_tree;
pub use self::kd_tree::*;

use std::marker::{Send, Sync};

pub trait Raytraceable : Send + Sync {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult;
}