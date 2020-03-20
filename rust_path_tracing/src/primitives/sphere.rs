use super::Raytraceable;
use crate::math::*;
use crate::ray::*;

pub struct Sphere {
    center: Vec3,
    radius: f32,
    radius_sqr: f32,

    material_id: usize,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_id: usize) -> Sphere {
        Sphere { center, radius, radius_sqr: radius * radius, material_id }
    }
}

impl Raytraceable for Sphere {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();

        let a = &self.center - &ray.source;
        //length(Direction * t + Source - Center) = radius
        // A = center - source
        //t^2 * dot(Direction, Direction) - 2 * t * dot(A, Direction) + dot(A, A) = Radius ^ 2
        //Direction is normalized => dot(Direction, Direction) = 1
        let half_second_k = -a.dot(&ray.direction);
        //Discriminant = second_k ^ 2 - 4 * first_k * third_k
        let discriminant = 4.0 * (half_second_k * half_second_k - (a.dot(&a) - self.radius_sqr));
        if discriminant < 0.0 {
            return result;
        }
        //roots are (-half_second_k * 2 +- sqrtD) / 2
        let d_sqrt = discriminant.sqrt();
        let t1 = -half_second_k + d_sqrt / 2.0;
        let t2 = -half_second_k - d_sqrt / 2.0;

        if t2 >= ray.min && t2 <= ray.max {
            result.t = t2;
        //result.normal_facing_outside = 1;
        } else if t1 >= ray.min && t1 <= ray.max {
            result.t = t1;
        //if we choose max value of t it means that ray is traced from inside
        //result.normal_facing_outside = -1;
        } else {
            return result;
        }

        result.point = &(result.t * &ray.direction) + &ray.source;
        result.normal = &(&result.point - &self.center) / self.radius;
        result.hit = true;
        result.material_id = self.material_id;

        result
    }
}
