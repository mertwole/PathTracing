use crate::vec3::*;

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    pub fn void() -> AABB {
        AABB {
            min: Vec3::zero(),
            max: Vec3::zero(),
        }
    }

    pub fn split(&self, plane_pos: &Vec3, plane_normal: &Vec3) -> (AABB, AABB) {
        let split_pos_along_normal = plane_pos * plane_normal;
        let one_minus_normal = &Vec3::new(1.0, 1.0, 1.0) - &plane_normal;
        (
            AABB::new(
                self.min.clone(),
                &split_pos_along_normal + &(&one_minus_normal * &self.max),
            ),
            AABB::new(
                &split_pos_along_normal + &(&one_minus_normal * &self.min),
                self.max.clone(),
            ),
        )
    }
}
