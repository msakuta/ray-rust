

use std::ops::{Add, AddAssign, Sub, Mul};

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


pub type Mat4 = [[f32; 4]; 3];

pub const MAT4IDENTITY: Mat4 = [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.]];

pub fn concat(m: &Mat4, v: &Vec3) -> Vec3{
    Vec3{
        x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3],
        y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3],
        z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3],
        reserved: 0.
    }
}


pub fn matcat(m1: &Mat4, m2: &Mat4) -> Mat4{
	let mut ret: Mat4 = MAT4IDENTITY;
	for i in 0..3 {
        for j in 0..4 {
            ret[i][j] =
                m1[i][0] * m2[0][j] +
                m1[i][1] * m2[1][j] +
                m1[i][2] * m2[2][j];
        }
    }
    ret
}


