use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::math::vec::*;

#[allow(dead_code)]
pub trait Vec2Swizzles {
    type T;

    fn x(&self) -> Self::T;
    fn y(&self) -> Self::T;

    fn xx(&self) -> [Self::T; 2];
    fn xy(&self) -> [Self::T; 2];
    fn yx(&self) -> [Self::T; 2];
    fn yy(&self) -> [Self::T; 2];
}

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec2f {
    pub data: [f32; 2],
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { data: [x, y] }
    }
}

impl Length<f32> for Vec2f {
    fn length(self) -> f32 {
        f32::sqrt((self.x() * self.x()) + (self.y() * self.y()))
    }
}

impl Distance<f32> for Vec2f {
    fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }
}

impl Normalized for Vec2f {
    fn normalized(self) -> Self {
        self / self.length()
    }
}

impl Reflect for Vec2f {
    fn reflect(incident: Self, normal: Self) -> Self {
        incident - (normal * 2.0 * Self::dot(incident, normal))
    }
}

impl Refract for Vec2f {
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

impl Dot<f32> for Vec2f {
    fn dot(a: Self, b: Self) -> f32 {
        (a.x() * b.x()) + (a.y() * b.y())
    }
}

impl Cross<f32> for Vec2f {
    fn cross(a: Self, b: Self) -> f32 {
        (a.x() * b.y()) - (a.y() * b.x())
    }
}

impl Min for Vec2f {
    fn min(a: Self, b: Self) -> Self {
        Self::new(f32::min(a.x(), b.x()), f32::min(a.y(), b.y()))
    }
}
impl Max for Vec2f {
    fn max(a: Self, b: Self) -> Self {
        Self::new(f32::max(a.x(), b.x()), f32::max(a.y(), b.y()))
    }
}

impl Powf<Self> for Vec2f {
    fn powf(a: Self, b: Self) -> Self {
        Self::new(f32::powf(a.x(), b.x()), f32::powf(a.y(), b.y()))
    }
}

impl Powf<f32> for Vec2f {
    fn powf(a: Self, b: f32) -> Self {
        Self::new(f32::powf(a.x(), b), f32::powf(a.y(), b))
    }
}

impl Abs for Vec2f {
    fn abs(self) -> Self {
        Self::new(f32::abs(self.x()), f32::abs(self.y()))
    }
}

impl Reversed for Vec2f {
    fn reversed(self) -> Self {
        Self::new(-self.x(), -self.y())
    }
}

impl Mix<Self> for Vec2f {
    fn mix(a: Self, b: Self, amount: Self) -> Self {
        Self::new(
            (a.x() * (1.0 - amount.x())) + b.x() * amount.x(),
            (a.y() * (1.0 - amount.y())) + b.y() * amount.y(),
        )
    }
}

impl Mix<f32> for Vec2f {
    fn mix(a: Self, b: Self, amount: f32) -> Self {
        Self::new(
            (a.x() * (1.0 - amount)) + b.x() * amount,
            (a.y() * (1.0 - amount)) + b.y() * amount,
        )
    }
}

impl Display for Vec2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value: ({}, {})\t|\tLength: {}",
            self.x(),
            self.y(),
            self.length()
        )
    }
}

impl From<f32> for Vec2f {
    fn from(value: f32) -> Self {
        Self::new(value, value)
    }
}

impl From<[f32; 2]> for Vec2f {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl Add for Vec2f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl AddAssign for Vec2f {
    fn add_assign(&mut self, rhs: Self) {
        self.data[0] += rhs.data[0];
        self.data[1] += rhs.data[1];
    }
}

impl Sub for Vec2f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        self.data[0] -= rhs.data[0];
        self.data[1] -= rhs.data[1];
    }
}

impl Mul for Vec2f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x() * rhs.x(), self.y() * rhs.y())
    }
}

impl Mul<f32> for Vec2f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x() * rhs, self.y() * rhs)
    }
}

impl MulAssign for Vec2f {
    fn mul_assign(&mut self, rhs: Self) {
        self.data[0] *= rhs.data[0];
        self.data[1] *= rhs.data[1];
    }
}

impl MulAssign<f32> for Vec2f {
    fn mul_assign(&mut self, rhs: f32) {
        self.data[0] *= rhs;
        self.data[1] *= rhs;
    }
}

impl Div for Vec2f {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x() / rhs.x(), self.y() / rhs.y())
    }
}

impl Div<f32> for Vec2f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x() / rhs, self.y() / rhs)
    }
}

impl DivAssign for Vec2f {
    fn div_assign(&mut self, rhs: Self) {
        self.data[0] /= rhs.data[0];
        self.data[1] /= rhs.data[1];
    }
}

impl DivAssign<f32> for Vec2f {
    fn div_assign(&mut self, rhs: f32) {
        self.data[0] /= rhs;
        self.data[1] /= rhs;
    }
}

impl Vec2Swizzles for Vec2f {
    type T = f32;

    fn x(&self) -> Self::T {
        return self.data[0];
    }

    fn y(&self) -> Self::T {
        return self.data[1];
    }

    fn xx(&self) -> [Self::T; 2] {
        return [self.data[0], self.data[0]];
    }

    fn xy(&self) -> [Self::T; 2] {
        return [self.data[0], self.data[1]];
    }

    fn yx(&self) -> [Self::T; 2] {
        return [self.data[1], self.data[0]];
    }

    fn yy(&self) -> [Self::T; 2] {
        return [self.data[1], self.data[1]];
    }
}
