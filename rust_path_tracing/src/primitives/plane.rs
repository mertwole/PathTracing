use crate::math::*;
use crate::ray::*;
use super::Raytraceable;

pub struct Plane{
    pub point : Vec3,
    pub normal : Vec3
}

impl Plane{
    pub fn new(point : Vec3, normal : Vec3) -> Plane{
        Plane{point, normal}
    }
}

impl Raytraceable for Plane{
    fn TraceRay(&self, ray : &Ray) -> RayTraceResult{
        let mut result = RayTraceResult::void();
    
        //plane equality:
        //Nx(x - x0) + Ny(y - y0) + Nz(z - z0) = 0
        //where N - normal vector to plane
        //V[0](x0, y0, z0) - any point on this plane
        //point on ray = t * Direction + source
        //   =>
        //t = Dot(N, V[0] - Source) / Dot(N, Direction)
        //Dot(N, Direction) == 0 when Normal is perpendicular to direction => Direction parrallel to plane
        let t = self.normal.dot(&(&self.point - &ray.source)) / self.normal.dot(&ray.direction);//dot(plane.normal, plane.point - ray.source) / dot(plane.normal, ray.direction);
        
        if t < ray.min || t > ray.max {		
            return result;
        }

        result.hit = true;
        result.point = &ray.source + &(&ray.direction * t);		
        result.normal = self.normal.clone();
        //result.normal_facing_outside = sign(dot(plane.normal, -ray.direction));
        result.t = t;

        result
    }
}