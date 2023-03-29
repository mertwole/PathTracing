use math::Vec3;

pub struct Ray {
    pub source: Vec3,
    pub direction: Vec3,

    pub min: f32,
    pub max: f32,
}

impl Ray {
    pub fn new(source: Vec3, direction: Vec3, min: f32, max: f32) -> Ray {
        Ray {
            source,
            direction,
            min,
            max,
        }
    }
}
