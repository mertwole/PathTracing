use std::ops;

pub const EPSILON : f32 = 0.0001;
pub const PI : f32 = 3.14159265359;
pub const INV_PI : f32 = 0.31830988618;

pub struct Math { }

impl Math{
    pub fn small_enought(x : f32) -> bool {
        x < std::f32::EPSILON * 10.0 && x > -std::f32::EPSILON * 10.0
    }
    
    pub fn min(a : f32, b : f32) -> f32 {
        if a < b { a } else { b } 
    }

    pub fn max(a : f32, b : f32) -> f32 {
        if a > b { a } else { b } 
    }

    pub fn min_triple(a : f32, b : f32, c : f32) -> f32{
        Math::min(a, Math::min(b, c))
    }

    pub fn max_triple(a : f32, b : f32, c : f32) -> f32{
        Math::max(a, Math::max(b, c))
    }
}

// region UVec2 
pub struct UVec2{
    pub x : usize,
    pub y : usize
}

impl UVec2{
    pub fn new(x : usize, y : usize) -> UVec2{
        UVec2 { x, y }
    }

    pub fn zero() -> UVec2{
        UVec2 { x : 0, y : 0 }
    }

    pub fn clone(&self) -> UVec2{
        UVec2 { x : self.x, y : self.y }
    }
}

impl ops::Add<&UVec2> for &UVec2 {
    type Output = UVec2;
    fn add(self, rhs: &UVec2) -> UVec2 {
        UVec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Sub<&UVec2> for &UVec2 {
    type Output = UVec2;
    fn sub(self, rhs: &UVec2) -> UVec2 {
        UVec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Mul<&UVec2> for &UVec2 {
    type Output = UVec2;
    fn mul(self, rhs: &UVec2) -> UVec2 {
        UVec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::Mul<usize> for &UVec2 {
    type Output = UVec2;
    fn mul(self, rhs: usize) -> UVec2 {
        UVec2::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<&UVec2> for usize {
    type Output = UVec2;
    fn mul(self, rhs: &UVec2) -> UVec2 {
        rhs * self
    }
}

impl ops::Div<&UVec2> for &UVec2 {
    type Output = UVec2;
    fn div(self, rhs: &UVec2) -> UVec2 {
        UVec2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::Div<usize> for &UVec2 {
    type Output = UVec2;
    fn div(self, rhs: usize) -> UVec2 {
        UVec2::new(self.x / rhs, self.y / rhs)
    }
}

// endregion

// region Color

pub struct Color24bpprgb{
    pub r : u8,
    pub g : u8,
    pub b : u8
}

impl Color24bpprgb{
    pub fn new(r : u8, g : u8, b : u8) -> Color24bpprgb{
        Color24bpprgb { r, g, b }
    }

    pub fn from_normalized(r : f32, g : f32, b : f32) -> Color24bpprgb{
        Color24bpprgb { r : (r * 255.0) as u8, g : (g * 255.0) as u8, b : (b * 255.0) as u8 }
    }

    pub fn zero() -> Color24bpprgb{
        Color24bpprgb { r : 0, g : 0, b : 0 }
    }

    pub fn clone(&self) -> Color24bpprgb{
        Color24bpprgb { r : self.r, g : self.g, b : self.b }
    }
}

// endregion

// region Vec2
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn zero() -> Vec2 {
        Vec2 { x: 0.0, y: 0.0 }
    }

    pub fn clone(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn equals(&self, rhs: &Vec2) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }

    pub fn sqr_length(&self) -> f32 {
        self.dot(self)
    }

    pub fn length(&self) -> f32 {
        self.sqr_length().sqrt()
    }

    pub fn normalized(&self) -> Vec2 {
        self / self.length()
    }

    pub fn dot(&self, rhs: &Vec2) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl ops::Add<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: &Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Sub<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: &Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Mul<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: &Vec2) -> Vec2 {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::Mul<f32> for &Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<&Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: &Vec2) -> Vec2 {
        rhs * self
    }
}

impl ops::Div<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn div(self, rhs: &Vec2) -> Vec2 {
        Vec2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::Div<f32> for &Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}
// endregion

// region Vec3
#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn new_xyz(xyz : f32) -> Vec3{
        Vec3 { x : xyz, y : xyz, z : xyz }
    }

    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn clone(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn sqr_length(&self) -> f32 {
        self.dot(self)
    }

    pub fn length(&self) -> f32 {
        self.sqr_length().sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        self / self.length()
    }

    pub fn dot(&self, rhs: &Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn reflect(&self, relative: &Vec3) -> Vec3 {
        self - &(2.0 * self.dot(relative) * relative)
    }

    pub fn refract(&self, normal : &Vec3, eta : f32) -> Option<Vec3> {
        let dot = normal.dot(self);
        let k = 1.0 - eta * eta * (1.0 - dot * dot);
        if k < 0.0 { return None; }
        Some(&(eta * self) - &(normal * (eta * dot + f32::sqrt(k))))
    }

    pub fn random_on_unit_sphere(rand_0 : f32, rand_1 : f32) -> Vec3 {
        let theta = rand_0 * PI * 2.0;
        let phi = f32::acos((2.0 * rand_1) - 1.0);
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::sin(phi) * f32::sin(theta);
        let z = f32::cos(phi);

        Vec3::new(x, y, z)
    }

    // Angle is in steradians.
    pub fn random_in_solid_angle(direction : &Vec3, angle : f32, rand_0 : f32, rand_1 : f32) -> Vec3 {
        let theta = rand_0 * PI * 2.0;
        let phi = f32::acos((1.0 - rand_1) * f32::cos(angle * 0.25) + rand_1);
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::sin(phi) * f32::sin(theta);
        let z = f32::cos(phi);
        let rand = Vec3::new(x, y, z);
        let rotation = Mat3::create_rotation_from_to(&Vec3::new(0.0, 0.0, 1.0), &direction);
        &rotation * &rand
    } 
}

impl ops::Add<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn add(self, rhs: &Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: &Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::Mul<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl ops::Mul<f32> for &Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::Mul<&Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        rhs * self
    }
}

impl ops::Div<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn div(self, rhs: &Vec3) -> Vec3 {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl ops::Div<f32> for &Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

// endregion

// region Mat3
pub struct Mat3 {
    row0: Vec3,
    row1: Vec3,
    row2: Vec3,
}

impl Mat3 {
    pub fn new(row0: Vec3, row1: Vec3, row2: Vec3) -> Mat3 {
        Mat3 { row0, row1, row2 }
    }

    pub fn identity() -> Mat3 {
        Mat3::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0)
        )
    }

    pub fn create_rotation_x(rotation : f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, cos, -sin),
            Vec3::new(0.0, sin, cos)
        )
    }

    pub fn create_rotation_y(rotation : f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(cos, 0.0, sin),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-sin, 0.0, cos)
        )
    }

    pub fn create_rotation_z(rotation : f32) -> Mat3 {
        let (sin, cos) = f32::sin_cos(rotation);

        Mat3::new(
            Vec3::new(cos, -sin, 0.0),
            Vec3::new(sin, cos, 0.0),
            Vec3::new(0.0, 0.0, 1.0)
        )
    }

    // From and to are normalized.
    pub fn create_rotation_from_to(from : &Vec3, to : &Vec3) -> Mat3 {
        let cross = from.cross(to);
        let cos = from.dot(to);

        let v = Mat3::new(
            Vec3::new(0.0, -cross.z, cross.y),
            Vec3::new(cross.z, 0.0, -cross.x),
            Vec3::new(-cross.y, cross.x, 0.0));

        &(&Mat3::identity() + &v) + &(&(&v * &v) * (1.0 / (1.0 + cos)))
    }
}

impl ops::Mul<&Vec3> for &Mat3 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        Vec3::new(
            rhs.dot(&self.row0),
            rhs.dot(&self.row1),
            rhs.dot(&self.row2),
        )
    }
}

impl ops::Mul<f32> for &Mat3 {
    type Output = Mat3;
    fn mul(self, rhs : f32) -> Mat3 {
        Mat3::new(&self.row0 * rhs, &self.row1 * rhs, &self.row2 * rhs)
    }
}

impl ops::Add<&Mat3> for &Mat3 {
    type Output = Mat3;
    fn add(self, rhs : &Mat3) -> Mat3 {
        Mat3::new(&self.row0 + &rhs.row0, &self.row1 + &rhs.row1, &self.row2 + &rhs.row2)
    }
}

impl ops::Mul<&Mat3> for &Mat3 {
    type Output = Mat3;
    fn mul(self, rhs: &Mat3) -> Mat3 {
        let col0 = Vec3::new(rhs.row0.x, rhs.row1.x, rhs.row2.x);
        let col1 = Vec3::new(rhs.row0.y, rhs.row1.y, rhs.row2.y);
        let col2 = Vec3::new(rhs.row0.z, rhs.row1.z, rhs.row2.z);

        Mat3::new(
            Vec3::new(self.row0.dot(&col0), self.row0.dot(&col1), self.row0.dot(&col2)),
            Vec3::new(self.row1.dot(&col0), self.row1.dot(&col1), self.row1.dot(&col2)),
            Vec3::new(self.row2.dot(&col0), self.row2.dot(&col1), self.row2.dot(&col2))
        )
    }
}


// endregion