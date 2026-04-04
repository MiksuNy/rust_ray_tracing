use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::math::{rand_f32_nd, vec::*};

#[allow(dead_code)]
pub trait Vec3Swizzles {
    type T;

    fn x(&self) -> Self::T;
    fn y(&self) -> Self::T;
    fn z(&self) -> Self::T;

    fn xx(&self) -> [Self::T; 2];
    fn xy(&self) -> [Self::T; 2];
    fn xz(&self) -> [Self::T; 2];
    fn yx(&self) -> [Self::T; 2];
    fn yy(&self) -> [Self::T; 2];
    fn yz(&self) -> [Self::T; 2];
    fn zx(&self) -> [Self::T; 2];
    fn zy(&self) -> [Self::T; 2];
    fn zz(&self) -> [Self::T; 2];

    fn xxx(&self) -> [Self::T; 3];
    fn xxy(&self) -> [Self::T; 3];
    fn xxz(&self) -> [Self::T; 3];
    fn yxx(&self) -> [Self::T; 3];
    fn yxy(&self) -> [Self::T; 3];
    fn yxz(&self) -> [Self::T; 3];
    fn zxx(&self) -> [Self::T; 3];
    fn zxy(&self) -> [Self::T; 3];
    fn zxz(&self) -> [Self::T; 3];
    fn xyx(&self) -> [Self::T; 3];
    fn xyy(&self) -> [Self::T; 3];
    fn xyz(&self) -> [Self::T; 3];
    fn yyx(&self) -> [Self::T; 3];
    fn yyy(&self) -> [Self::T; 3];
    fn yyz(&self) -> [Self::T; 3];
    fn zyx(&self) -> [Self::T; 3];
    fn zyy(&self) -> [Self::T; 3];
    fn zyz(&self) -> [Self::T; 3];
    fn xzx(&self) -> [Self::T; 3];
    fn xzy(&self) -> [Self::T; 3];
    fn xzz(&self) -> [Self::T; 3];
    fn yzx(&self) -> [Self::T; 3];
    fn yzy(&self) -> [Self::T; 3];
    fn yzz(&self) -> [Self::T; 3];
    fn zzx(&self) -> [Self::T; 3];
    fn zzy(&self) -> [Self::T; 3];
    fn zzz(&self) -> [Self::T; 3];
}

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec3f {
    pub data: [f32; 3],
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { data: [x, y, z] }
    }

    pub fn rand_in_unit_sphere(input: &mut u32) -> Self {
        Self::new(rand_f32_nd(input), rand_f32_nd(input), rand_f32_nd(input)).normalized()
    }

    pub fn rand_in_unit_hemisphere(input: &mut u32, normal: Self) -> Self {
        let unit_sphere = Self::rand_in_unit_sphere(input);
        if Self::dot(unit_sphere, normal) < 0.0 {
            return unit_sphere.reversed();
        } else {
            return unit_sphere;
        }
    }

    // https://gamedev.stackexchange.com/a/194038
    pub fn linear_to_srgb(linear: Self) -> Self {
        let cutoff = Self::new(
            ((linear.x() < 0.0031308) as u32) as f32,
            ((linear.y() < 0.0031308) as u32) as f32,
            ((linear.z() < 0.0031308) as u32) as f32,
        );
        let higher =
            Self::from(1.055) * Self::powf(linear, Self::from(1.0 / 2.4)) - Self::from(0.055);
        let lower = linear * Self::from(12.92);
        return Self::mix(higher, lower, cutoff);
    }
}

impl Length<f32> for Vec3f {
    fn length(self) -> f32 {
        f32::sqrt((self.x() * self.x()) + (self.y() * self.y()) + (self.z() * self.z()))
    }
}

impl Distance<f32> for Vec3f {
    fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }
}

impl Normalized for Vec3f {
    fn normalized(self) -> Self {
        self / self.length()
    }
}

impl Reflect for Vec3f {
    fn reflect(incident: Self, normal: Self) -> Self {
        incident - (normal * 2.0 * Self::dot(incident, normal))
    }
}

impl Refract for Vec3f {
    fn refract(incident: Self, normal: Self, eta: f32) -> Self {
        let k =
            1.0 - (eta * eta) * (1.0 - (Self::dot(normal, incident) * Self::dot(normal, incident)));
        if k < 0.0 {
            return Self::from(0.0);
        } else {
            let eta_dot_n_i = eta * Self::dot(normal, incident);
            return (incident * eta) - (Self::from(eta_dot_n_i + f32::sqrt(k)) * normal);
        }
    }
}

impl Dot<f32> for Vec3f {
    fn dot(a: Self, b: Self) -> f32 {
        (a.x() * b.x()) + (a.y() * b.y()) + (a.z() * b.z())
    }
}

impl Cross<Self> for Vec3f {
    fn cross(a: Self, b: Self) -> Self {
        Self::new(
            (a.y() * b.z()) - (a.z() * b.y()),
            (a.z() * b.x()) - (a.x() * b.z()),
            (a.x() * b.y()) - (a.y() * b.x()),
        )
    }
}

impl Min for Vec3f {
    fn min(a: Self, b: Self) -> Self {
        Self::new(
            f32::min(a.x(), b.x()),
            f32::min(a.y(), b.y()),
            f32::min(a.z(), b.z()),
        )
    }
}
impl Max for Vec3f {
    fn max(a: Self, b: Self) -> Self {
        Self::new(
            f32::max(a.x(), b.x()),
            f32::max(a.y(), b.y()),
            f32::max(a.z(), b.z()),
        )
    }
}

impl Powf<Self> for Vec3f {
    fn powf(a: Self, b: Self) -> Self {
        Self::new(
            f32::powf(a.x(), b.x()),
            f32::powf(a.y(), b.y()),
            f32::powf(a.z(), b.z()),
        )
    }
}

impl Powf<f32> for Vec3f {
    fn powf(a: Self, b: f32) -> Self {
        Self::new(
            f32::powf(a.x(), b),
            f32::powf(a.y(), b),
            f32::powf(a.z(), b),
        )
    }
}

impl Abs for Vec3f {
    fn abs(self) -> Self {
        Self::new(f32::abs(self.x()), f32::abs(self.y()), f32::abs(self.z()))
    }
}

impl Reversed for Vec3f {
    fn reversed(self) -> Self {
        Self::new(-self.x(), -self.y(), -self.z())
    }
}

impl Mix<Self> for Vec3f {
    fn mix(a: Self, b: Self, amount: Self) -> Self {
        Self::new(
            (a.x() * (1.0 - amount.x())) + b.x() * amount.x(),
            (a.y() * (1.0 - amount.y())) + b.y() * amount.y(),
            (a.z() * (1.0 - amount.z())) + b.z() * amount.z(),
        )
    }
}

impl Mix<f32> for Vec3f {
    fn mix(a: Self, b: Self, amount: f32) -> Self {
        Self::new(
            (a.x() * (1.0 - amount)) + b.x() * amount,
            (a.y() * (1.0 - amount)) + b.y() * amount,
            (a.z() * (1.0 - amount)) + b.z() * amount,
        )
    }
}

impl Display for Vec3f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value: ({}, {}, {})\t|\tLength: {}",
            self.x(),
            self.y(),
            self.z(),
            self.length()
        )
    }
}

impl From<f32> for Vec3f {
    fn from(value: f32) -> Self {
        Self::new(value, value, value)
    }
}

impl From<[f32; 3]> for Vec3f {
    fn from(value: [f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<[u8; 3]> for Vec3f {
    fn from(color: [u8; 3]) -> Self {
        Self::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        )
    }
}

impl From<[u8; 4]> for Vec3f {
    fn from(color: [u8; 4]) -> Self {
        Self::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        )
    }
}

impl From<Vec3f> for [u8; 3] {
    fn from(vector: Vec3f) -> Self {
        [
            f32::floor(vector.x() * 255.0).clamp(0.0, 255.0) as u8,
            f32::floor(vector.y() * 255.0).clamp(0.0, 255.0) as u8,
            f32::floor(vector.z() * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }
}

impl Add for Vec3f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl AddAssign for Vec3f {
    fn add_assign(&mut self, rhs: Self) {
        self.data[0] += rhs.data[0];
        self.data[1] += rhs.data[1];
        self.data[2] += rhs.data[2];
    }
}

impl Sub for Vec3f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

impl SubAssign for Vec3f {
    fn sub_assign(&mut self, rhs: Self) {
        self.data[0] -= rhs.data[0];
        self.data[1] -= rhs.data[1];
        self.data[2] -= rhs.data[2];
    }
}

impl Mul for Vec3f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x() * rhs.x(), self.y() * rhs.y(), self.z() * rhs.z())
    }
}

impl Mul<f32> for Vec3f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

impl MulAssign for Vec3f {
    fn mul_assign(&mut self, rhs: Self) {
        self.data[0] *= rhs.data[0];
        self.data[1] *= rhs.data[1];
        self.data[2] *= rhs.data[2];
    }
}

impl MulAssign<f32> for Vec3f {
    fn mul_assign(&mut self, rhs: f32) {
        self.data[0] *= rhs;
        self.data[1] *= rhs;
        self.data[2] *= rhs;
    }
}

impl Div for Vec3f {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x() / rhs.x(), self.y() / rhs.y(), self.z() / rhs.z())
    }
}

impl Div<f32> for Vec3f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x() / rhs, self.y() / rhs, self.z() / rhs)
    }
}

impl DivAssign for Vec3f {
    fn div_assign(&mut self, rhs: Self) {
        self.data[0] /= rhs.data[0];
        self.data[1] /= rhs.data[1];
        self.data[2] /= rhs.data[2];
    }
}

impl DivAssign<f32> for Vec3f {
    fn div_assign(&mut self, rhs: f32) {
        self.data[0] /= rhs;
        self.data[1] /= rhs;
        self.data[2] /= rhs;
    }
}

impl Vec3Swizzles for Vec3f {
    type T = f32;

    fn x(&self) -> Self::T {
        return self.data[0];
    }

    fn y(&self) -> Self::T {
        return self.data[1];
    }

    fn z(&self) -> Self::T {
        return self.data[2];
    }

    fn xx(&self) -> [Self::T; 2] {
        return [self.data[0], self.data[0]];
    }

    fn xy(&self) -> [Self::T; 2] {
        return [self.data[0], self.data[1]];
    }

    fn xz(&self) -> [Self::T; 2] {
        return [self.data[0], self.data[2]];
    }

    fn yx(&self) -> [Self::T; 2] {
        return [self.data[1], self.data[0]];
    }

    fn yy(&self) -> [Self::T; 2] {
        return [self.data[1], self.data[1]];
    }

    fn yz(&self) -> [Self::T; 2] {
        return [self.data[1], self.data[2]];
    }

    fn zx(&self) -> [Self::T; 2] {
        return [self.data[2], self.data[0]];
    }

    fn zy(&self) -> [Self::T; 2] {
        return [self.data[2], self.data[1]];
    }

    fn zz(&self) -> [Self::T; 2] {
        return [self.data[2], self.data[2]];
    }

    fn xxx(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[0], self.data[0]];
    }

    fn xxy(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[0], self.data[1]];
    }

    fn xxz(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[0], self.data[2]];
    }

    fn yxx(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[0], self.data[0]];
    }

    fn yxy(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[0], self.data[1]];
    }

    fn yxz(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[0], self.data[2]];
    }

    fn zxx(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[0], self.data[0]];
    }

    fn zxy(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[0], self.data[1]];
    }

    fn zxz(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[0], self.data[2]];
    }

    fn xyx(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[1], self.data[0]];
    }

    fn xyy(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[1], self.data[1]];
    }

    fn xyz(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[1], self.data[2]];
    }

    fn yyx(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[1], self.data[0]];
    }

    fn yyy(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[1], self.data[1]];
    }

    fn yyz(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[1], self.data[2]];
    }

    fn zyx(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[1], self.data[0]];
    }

    fn zyy(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[1], self.data[1]];
    }

    fn zyz(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[1], self.data[2]];
    }

    fn xzx(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[2], self.data[0]];
    }

    fn xzy(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[2], self.data[1]];
    }

    fn xzz(&self) -> [Self::T; 3] {
        return [self.data[0], self.data[2], self.data[2]];
    }

    fn yzx(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[2], self.data[0]];
    }

    fn yzy(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[2], self.data[1]];
    }

    fn yzz(&self) -> [Self::T; 3] {
        return [self.data[1], self.data[2], self.data[2]];
    }

    fn zzx(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[2], self.data[0]];
    }

    fn zzy(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[2], self.data[1]];
    }

    fn zzz(&self) -> [Self::T; 3] {
        return [self.data[2], self.data[2], self.data[2]];
    }
}
