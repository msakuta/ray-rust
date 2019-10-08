

use std::ops::{Add, AddAssign, Sub, Mul};
use std::convert::Into;

#[derive(Clone, Debug, Copy)]
pub struct Vec3{
	pub x: f32,
	pub y: f32,
	pub z: f32,
	reserved: f32,
}

impl Vec3{
    pub fn new(x: f32, y: f32, z: f32) -> Vec3{
        Vec3{x, y, z, reserved: 1.}
    }

    pub fn zero() -> Self{
        Self::new(0., 0., 0.)
    }

    pub fn dot(&self, b: &Self) -> f32 {
        self.x*b.x
         + self.y*b.y
         + self.z*b.z
    } 

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }

    pub fn normalize(&mut self){
        let len = self.len();
        self.scale(1. / len);
    }

    pub fn scaled(&self, o: f32) -> Self{
        Self::new(self.x * o, self.y * o, self.z * o)
    }

    pub fn scale(&mut self, o: f32){
        self.x *= o;
        self.y *= o;
        self.z *= o;
    }
}

// It's a shame that we cannot omit '&' in front of Vec3 object
// if we want to use multiplication operator (*).
// Another option is to call like v1.mul(v2), but it's ugly too.
impl Mul<f32> for &Vec3{
    type Output = Vec3;

    fn mul(self, o: f32) -> Vec3{
        Vec3::new(self.x * o, self.y * o, self.z * o)
    }
}

impl Add for &Vec3{
    type Output = Vec3;

    fn add(self, o: Self) -> Vec3{
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl AddAssign for Vec3{
    fn add_assign(&mut self, o: Vec3){
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}

impl Sub for &Vec3{
    type Output = Vec3;

    fn sub(self, o: Self) -> Vec3{
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

// We cannot easily define Mat4 class with all the operator overloading,
// so we give up and delegate to vecmath.
impl Into<vecmath::Vector3<f32>> for Vec3{
    fn into(self) -> vecmath::Vector3<f32> {
        [self.x, self.y, self.z]
    }
}

impl From<vecmath::Vector3<f32>> for Vec3{
    fn from(v: vecmath::Vector3<f32>) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

