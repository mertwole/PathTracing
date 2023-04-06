use math::{Mat4, Vec3, Vec4};

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

    pub fn apply_transform(&self, transform: &Mat4) -> Ray {
        let source = Vec4::from_vec3(self.source);
        let min_point = Vec4::from_vec3(self.min * self.direction + self.source);
        let max_point = Vec4::from_vec3(self.max * self.direction + self.source);

        let source = transform * source;
        let min_point = transform * min_point;
        let max_point = transform * max_point;
        let direction = &transform.normal_matrix() * self.direction;

        Ray {
            source,
            direction,
            min: (min_point - source).length(),
            max: (max_point - source).length(),
        }
    }
}
