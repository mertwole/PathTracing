use crate::vec3::*;

pub struct AABB{
    min : Vec3,
    max : Vec3
}

impl AABB{
    pub fn new(min : Vec3, max : Vec3) -> AABB{
        AABB { min, max } 
    }
}