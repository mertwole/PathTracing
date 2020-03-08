use std::ops;

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
