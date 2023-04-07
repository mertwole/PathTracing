use serde::{Deserialize, Serialize};
use std::ops;

pub const EPSILON: f32 = 0.0001;
pub const PI: f32 = 3.14159265359;
pub const INV_PI: f32 = 0.31830988618;

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
#[serde(expecting = "expecting [<x>, <y>] array")]
pub struct UVec2 {
    pub x: usize,
    pub y: usize,
}

impl UVec2 {
    pub fn new(x: usize, y: usize) -> UVec2 {
        UVec2 { x, y }
    }
}

impl ops::Add<UVec2> for UVec2 {
    type Output = UVec2;
    fn add(self, rhs: UVec2) -> UVec2 {
        UVec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Sub<UVec2> for UVec2 {
    type Output = UVec2;
    fn sub(self, rhs: UVec2) -> UVec2 {
        UVec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Mul<UVec2> for UVec2 {
    type Output = UVec2;
    fn mul(self, rhs: UVec2) -> UVec2 {
        UVec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::Mul<usize> for UVec2 {
    type Output = UVec2;
    fn mul(self, rhs: usize) -> UVec2 {
        UVec2::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<UVec2> for usize {
    type Output = UVec2;
    fn mul(self, rhs: UVec2) -> UVec2 {
        rhs * self
    }
}

impl ops::Div<UVec2> for UVec2 {
    type Output = UVec2;
    fn div(self, rhs: UVec2) -> UVec2 {
        UVec2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::Div<usize> for UVec2 {
    type Output = UVec2;
    fn div(self, rhs: usize) -> UVec2 {
        UVec2::new(self.x / rhs, self.y / rhs)
    }
}

#[derive(Clone, Copy, Default)]
pub struct HdrColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl HdrColor {
    pub fn from_vec3(vec3: Vec3) -> HdrColor {
        HdrColor {
            r: vec3.x,
            g: vec3.y,
            b: vec3.z,
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Color24bpprgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color24bpprgb {
    pub fn new(r: u8, g: u8, b: u8) -> Color24bpprgb {
        Color24bpprgb { r, g, b }
    }

    pub fn from_hdr_tone_mapped(hdr: HdrColor) -> Color24bpprgb {
        let hdr = hdr.to_vec3();
        let tone_mapped = hdr / (hdr + Vec3::new_xyz(1.0));
        Color24bpprgb::from_normalized(tone_mapped.x, tone_mapped.y, tone_mapped.z)
    }

    pub fn from_normalized(r: f32, g: f32, b: f32) -> Color24bpprgb {
        Color24bpprgb {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
#[serde(expecting = "expecting [<x>, <y>] array")]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn sqr_length(&self) -> f32 {
        self.dot(*self)
    }

    pub fn length(&self) -> f32 {
        self.sqr_length().sqrt()
    }

    pub fn normalized(&self) -> Vec2 {
        *self / self.length()
    }

    pub fn dot(&self, rhs: Vec2) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        rhs * self
    }
}

impl ops::Div<Vec2> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

struct Basis {
    pub i: Vec3,
    pub j: Vec3,
    pub k: Vec3,
}

impl Basis {
    fn build_orthonormal_by_j(j: Vec3) -> Basis {
        let i = if j.y.abs() + j.z.abs() < EPSILON {
            Vec3::new(-j.y, j.x, 0.0)
        } else {
            Vec3::new(0.0, j.z, -j.y)
        };
        let k = j.cross(i);
        Basis { i, j, k }
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
#[serde(expecting = "expecting [<x>, <y>, <z>] array")]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn new_xyz(xyz: f32) -> Vec3 {
        Vec3 {
            x: xyz,
            y: xyz,
            z: xyz,
        }
    }

    pub fn sqr_length(&self) -> f32 {
        self.dot(*self)
    }

    pub fn length(&self) -> f32 {
        self.sqr_length().sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        *self / self.length()
    }

    pub fn dot(&self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn reflect(&self, relative: Vec3) -> Vec3 {
        *self - 2.0 * self.dot(relative) * relative
    }

    pub fn refract(&self, normal: Vec3, eta: f32) -> Option<Vec3> {
        let dot = normal.dot(*self);
        let k = 1.0 - eta * eta * (1.0 - dot * dot);
        if k < 0.0 {
            return None;
        }
        Some(eta * *self - normal * (eta * dot + k.sqrt()))
    }

    pub fn random_on_unit_sphere(rand_0: f32, rand_1: f32) -> Vec3 {
        let theta = rand_0 * PI * 2.0;
        let phi = f32::acos((2.0 * rand_1) - 1.0);
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::sin(phi) * f32::sin(theta);
        let z = f32::cos(phi);

        Vec3::new(x, y, z)
    }

    pub fn random_on_hemisphere(rand_0: f32, rand_1: f32, normal: Vec3) -> Vec3 {
        let rand = Self::random_on_unit_sphere(rand_0, rand_1);
        if rand.dot(normal) < 0.0 {
            rand * -1.0
        } else {
            rand
        }
    }

    pub fn cosine_weighted_random_on_hemisphere(rand_0: f32, rand_1: f32, normal: Vec3) -> Vec3 {
        let rand_0 = f32::cos(PI * 0.5 * (1.0 - rand_0));
        let theta = rand_1 * PI * 2.0;

        let basis = Basis::build_orthonormal_by_j(normal);
        let r = rand_0.sqrt();
        // Without EPSILON ray goes under the surface sometimes.
        let rand = basis.j * (1.0 - rand_0 + EPSILON);
        rand + r * (basis.i * f32::cos(theta) + basis.k * f32::sin(theta))
    }

    // Angle is in steradians.
    pub fn random_in_solid_angle(direction: Vec3, angle: f32, rand_0: f32, rand_1: f32) -> Vec3 {
        let theta = rand_0 * PI * 2.0;
        let phi = f32::acos((1.0 - rand_1) * f32::cos(angle * 0.25) + rand_1);
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::sin(phi) * f32::sin(theta);
        let z = f32::cos(phi);
        let basis = Basis::build_orthonormal_by_j(direction);
        x * basis.i + y * basis.j + z * basis.k
    }
}

impl Into<Vec3> for [f32; 3] {
    fn into(self) -> Vec3 {
        Vec3::new(self[0], self[1], self[2])
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(expecting = "expecting [<row0>, <row1>, <row2>] array")]
pub struct Mat3 {
    pub row0: Vec3,
    pub row1: Vec3,
    pub row2: Vec3,
}

impl Default for Mat3 {
    fn default() -> Mat3 {
        Mat3::identity()
    }
}

impl Mat3 {
    pub fn new(row0: Vec3, row1: Vec3, row2: Vec3) -> Mat3 {
        Mat3 { row0, row1, row2 }
    }

    pub fn identity() -> Mat3 {
        Mat3::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
    }

    pub fn create_rotation_x(rotation: f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, cos, -sin),
            Vec3::new(0.0, sin, cos),
        )
    }

    pub fn create_rotation_y(rotation: f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(cos, 0.0, sin),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-sin, 0.0, cos),
        )
    }

    pub fn create_rotation_z(rotation: f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(cos, -sin, 0.0),
            Vec3::new(sin, cos, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
    }

    pub fn det(&self) -> f32 {
        self.row0.x * (self.row1.y * self.row2.z - self.row2.y * self.row1.z)
            - self.row0.y * (self.row1.x * self.row2.z - self.row1.z * self.row2.x)
            + self.row0.z * (self.row1.x * self.row2.y - self.row1.y * self.row2.x)
    }

    pub fn transponsed_inverse(&self) -> Mat3 {
        Mat3 {
            row0: Vec3::new(
                self.row1.y * self.row2.z - self.row2.y * self.row1.z,
                self.row1.z * self.row2.x - self.row1.x * self.row2.z,
                self.row1.x * self.row2.y - self.row2.x * self.row1.y,
            ),
            row1: Vec3::new(
                self.row0.z * self.row2.y - self.row0.y * self.row2.z,
                self.row0.x * self.row2.z - self.row0.z * self.row2.x,
                self.row2.x * self.row0.y - self.row0.x * self.row2.y,
            ),
            row2: Vec3::new(
                self.row0.y * self.row1.z - self.row0.z * self.row1.y,
                self.row1.x * self.row0.z - self.row0.x * self.row1.z,
                self.row0.x * self.row1.y - self.row1.x * self.row0.y,
            ),
        } * (1.0 / self.det())
    }
}

impl ops::Mul<Vec3> for &Mat3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(rhs.dot(self.row0), rhs.dot(self.row1), rhs.dot(self.row2))
    }
}

impl ops::Mul<f32> for Mat3 {
    type Output = Mat3;
    fn mul(self, rhs: f32) -> Mat3 {
        Mat3::new(self.row0 * rhs, self.row1 * rhs, self.row2 * rhs)
    }
}

impl ops::Add<Mat3> for Mat3 {
    type Output = Mat3;
    fn add(self, rhs: Mat3) -> Mat3 {
        Mat3::new(
            self.row0 + rhs.row0,
            self.row1 + rhs.row1,
            self.row2 + rhs.row2,
        )
    }
}

impl ops::Mul<Mat3> for Mat3 {
    type Output = Mat3;
    fn mul(self, rhs: Mat3) -> Mat3 {
        let col0 = Vec3::new(rhs.row0.x, rhs.row1.x, rhs.row2.x);
        let col1 = Vec3::new(rhs.row0.y, rhs.row1.y, rhs.row2.y);
        let col2 = Vec3::new(rhs.row0.z, rhs.row1.z, rhs.row2.z);

        Mat3::new(
            Vec3::new(
                self.row0.dot(col0),
                self.row0.dot(col1),
                self.row0.dot(col2),
            ),
            Vec3::new(
                self.row1.dot(col0),
                self.row1.dot(col1),
                self.row1.dot(col2),
            ),
            Vec3::new(
                self.row2.dot(col0),
                self.row2.dot(col1),
                self.row2.dot(col2),
            ),
        )
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
#[serde(expecting = "expecting [<x>, <y>, <z>, <w>] array")]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    pub fn from_vec3(vec3: Vec3) -> Vec4 {
        Vec4 {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
            w: 1.0,
        }
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn dot(self, other: Vec4) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(expecting = "expecting [<row0>, <row1>, <row2>, <row3>] array")]
pub struct Mat4 {
    pub row0: Vec4,
    pub row1: Vec4,
    pub row2: Vec4,
    pub row3: Vec4,
}

impl Mat4 {
    pub fn normal_matrix(&self) -> Mat3 {
        let upper_left = Mat3::new(
            self.row0.to_vec3(),
            self.row1.to_vec3(),
            self.row2.to_vec3(),
        );

        upper_left.transponsed_inverse()
    }

    fn get_minor(&self, row: usize, column: usize) -> f32 {
        let rows: Vec<_> = (0..4)
            .filter(|&r| r != row)
            .map(|row| match row {
                0 => self.row0,
                1 => self.row1,
                2 => self.row2,
                3 => self.row3,
                _ => unreachable!(),
            })
            .map(|row| match column {
                0 => Vec3::new(row.y, row.z, row.w),
                1 => Vec3::new(row.x, row.z, row.w),
                2 => Vec3::new(row.x, row.y, row.w),
                3 => Vec3::new(row.x, row.y, row.z),
                _ => unreachable!(),
            })
            .collect();

        Mat3::new(rows[0], rows[1], rows[2]).det()
    }

    pub fn transponsed_inverse(&self) -> Mat4 {
        let cofactor_matrix = Mat4 {
            row0: Vec4 {
                x: self.get_minor(0, 0),
                y: -self.get_minor(0, 1),
                z: self.get_minor(0, 2),
                w: -self.get_minor(0, 3),
            },
            row1: Vec4 {
                x: -self.get_minor(1, 0),
                y: self.get_minor(1, 1),
                z: -self.get_minor(1, 2),
                w: self.get_minor(1, 3),
            },
            row2: Vec4 {
                x: self.get_minor(2, 0),
                y: -self.get_minor(2, 1),
                z: self.get_minor(2, 2),
                w: -self.get_minor(2, 3),
            },
            row3: Vec4 {
                x: -self.get_minor(3, 0),
                y: self.get_minor(3, 1),
                z: -self.get_minor(3, 2),
                w: self.get_minor(3, 3),
            },
        };
        let det = self.row0.dot(cofactor_matrix.row0);
        cofactor_matrix * (1.0 / det)
    }

    pub fn transponse(self) -> Mat4 {
        Mat4 {
            row0: Vec4 {
                x: self.row0.x,
                y: self.row1.x,
                z: self.row2.x,
                w: self.row3.x,
            },
            row1: Vec4 {
                x: self.row0.y,
                y: self.row1.y,
                z: self.row2.y,
                w: self.row3.y,
            },
            row2: Vec4 {
                x: self.row0.z,
                y: self.row1.z,
                z: self.row2.z,
                w: self.row3.z,
            },
            row3: Vec4 {
                x: self.row0.w,
                y: self.row1.w,
                z: self.row2.w,
                w: self.row3.w,
            },
        }
    }

    pub fn inverse(&self) -> Mat4 {
        self.transponsed_inverse().transponse()
    }
}

impl ops::Mul<Vec4> for &Mat4 {
    type Output = Vec3;
    fn mul(self, rhs: Vec4) -> Vec3 {
        let w = rhs.dot(self.row3);
        Vec3::new(rhs.dot(self.row0), rhs.dot(self.row1), rhs.dot(self.row2)) / w
    }
}

impl ops::Mul<f32> for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: f32) -> Mat4 {
        Mat4 {
            row0: self.row0 * rhs,
            row1: self.row1 * rhs,
            row2: self.row2 * rhs,
            row3: self.row3 * rhs,
        }
    }
}
