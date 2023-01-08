use crate::aabb::*;
use math::Vec3;

pub struct Triangle {
    pub points: [Vec3; 3],
}

impl Triangle {
    pub fn check_with_aabb(&self, aabb: &AABB) -> bool {
        let aabb_vertices: [Vec3; 8] = [
            aabb.min,
            Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            aabb.max,
        ];
        //facets are: 0-1-2-3||4-5-6-7||1-2-7-6||2-3-4-7||0-3-4-5||0-1-6-5

        //plane equality is normal.x * x + normal.y * y + normal.z * z + d = 0
        let triangle_normal =
            (self.points[0] - self.points[1]).cross(self.points[0] - self.points[2]);
        let triangle_d = -triangle_normal.dot(self.points[0]);

        let mut intersection = false;

        let mut box_sign = 0;
        for i in 0..8 {
            let i_vert_side = triangle_normal.dot(aabb_vertices[i]) + triangle_d;
            if small_enought(i_vert_side) {
                continue;
            }
            if box_sign == 0 {
                box_sign = if i_vert_side > 0.0 { 1 } else { -1 };
                continue;
            }
            if (i_vert_side > 0.0 && box_sign == -1) || (i_vert_side < 0.0 && box_sign != -1) {
                intersection = true;
                break;
            }
        }

        if !intersection {
            return false;
        }

        let check_box_side = |point_ids: [usize; 3], opposite_side_point_id: usize| {
            let normal = (aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[1]])
                .cross(aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[2]]);
            let d = -normal.dot(aabb_vertices[point_ids[0]]);

            let triangle_side = if normal.dot(aabb_vertices[opposite_side_point_id]) + d < 0.0 {
                1
            } else {
                -1
            };
            for i in 0..3 {
                let triangle_dot_side = normal.dot(self.points[i]) + d;
                if small_enought(triangle_dot_side) {
                    continue;
                }
                let triangle_dot_side_i = if triangle_dot_side > 0.0 { 1 } else { -1 };
                if triangle_side != triangle_dot_side_i {
                    return true;
                }
            }

            false
        };

        if !check_box_side([0, 1, 2], 4) {
            return false;
        }
        if !check_box_side([4, 5, 6], 0) {
            return false;
        }
        if !check_box_side([1, 2, 7], 0) {
            return false;
        }
        if !check_box_side([2, 3, 4], 0) {
            return false;
        }
        if !check_box_side([0, 3, 4], 7) {
            return false;
        }
        if !check_box_side([0, 1, 6], 7) {
            return false;
        }

        true
    }
}

fn small_enought(a: f32) -> bool {
    a < std::f32::EPSILON * 10.0 && a > -std::f32::EPSILON * 10.0
}
