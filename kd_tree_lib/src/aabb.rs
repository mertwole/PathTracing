extern crate math_lib;
use math_lib::Vec3;

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
}
