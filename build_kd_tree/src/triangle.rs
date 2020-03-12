use crate::vec3::*;

pub struct Triangle {
    pub points: [Vec3; 3],
}

impl Triangle {
    pub fn new(points: [Vec3; 3]) -> Triangle {
        Triangle { points }
    }
}
