use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec3f {
    pub data: [f32; 3],
}

#[allow(dead_code)]
impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { data: [x, y, z] };
    }

    pub fn length(self) -> f32 {
        return f32::sqrt((self.x() * self.x()) + (self.y() * self.y()) + (self.z() * self.z()));
    }

    pub fn distance(a: Self, b: Self) -> f32 {
        return Self::sub(a, b).length();
    }

    pub fn normalized(self) -> Self {
        return Self {
            data: [
                self.x() / self.length(),
                self.y() / self.length(),
                self.z() / self.length(),
            ],
        };
    }

    pub fn reflect(incident: Self, normal: Self) -> Self {
        return incident - (normal * 2.0 * Self::dot(incident, normal));
    }

    /// eta = ratio of indices of refraction
    pub fn refract(incident: Self, normal: Self, eta: f32) -> Self {
        let k =
            1.0 - (eta * eta) * (1.0 - (Self::dot(normal, incident) * Self::dot(normal, incident)));
        if k < 0.0 {
            return Self::new(0.0, 0.0, 0.0);
        } else {
            let eta_dot_n_i = eta * Self::dot(normal, incident);
            return (incident * eta) - (Self::from(eta_dot_n_i + f32::sqrt(k)) * normal);
        }
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        return (a.x() * b.x()) + (a.y() * b.y()) + (a.z() * b.z());
    }

    pub fn cross(a: Self, b: Self) -> Self {
        return Self {
            data: [
                (a.y() * b.z()) - (a.z() * b.y()),
                (a.z() * b.x()) - (a.x() * b.z()),
                (a.x() * b.y()) - (a.y() * b.x()),
            ],
        };
    }

    pub fn min(a: Self, b: Self) -> Self {
        return Self {
            data: [
                f32::min(a.x(), b.x()),
                f32::min(a.y(), b.y()),
                f32::min(a.z(), b.z()),
            ],
        };
    }

    pub fn max(a: Self, b: Self) -> Self {
        return Self {
            data: [
                f32::max(a.x(), b.x()),
                f32::max(a.y(), b.y()),
                f32::max(a.z(), b.z()),
            ],
        };
    }

    pub fn abs(self) -> Self {
        return Self {
            data: [f32::abs(self.x()), f32::abs(self.y()), f32::abs(self.z())],
        };
    }

    pub fn reversed(self) -> Self {
        return Self {
            data: [-self.x(), -self.y(), -self.z()],
        };
    }

    pub fn lerp(a: Self, b: Self, amount: f32) -> Self {
        return (a * (1.0 - amount)) + b * amount;
    }

    fn xor_shift(input: &mut u32) -> u32 {
        let mut x: u32 = *input;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *input = x;
        return x;
    }

    /// Returns a random f32 in the range 0.0 - 1.0
    pub fn rand_f32(input: &mut u32) -> f32 {
        return Self::xor_shift(input) as f32 / u32::MAX as f32;
    }

    fn rand_f32_nd(input: &mut u32) -> f32 {
        let theta = 6.283185 * Self::rand_f32(input);
        let rho = f32::sqrt(-2.0 * f32::log10(Self::rand_f32(input)));
        return rho * f32::cos(theta);
    }

    pub fn rand_in_unit_sphere(input: &mut u32) -> Self {
        return Self {
            data: [
                Self::rand_f32_nd(input),
                Self::rand_f32_nd(input),
                Self::rand_f32_nd(input),
            ],
        }
        .normalized();
    }

    pub fn rand_in_unit_hemisphere(input: &mut u32, normal: Self) -> Self {
        let unit_sphere = Self::rand_in_unit_sphere(input);
        if Self::dot(unit_sphere, normal) < 0.0 {
            return unit_sphere.reversed();
        } else {
            return unit_sphere;
        }
    }

    pub fn linear_to_gamma(linear: Self) -> Self {
        let mut gamma = Self::new(0.0, 0.0, 0.0);
        for i in 0..3 {
            if linear.data[i] > 0.0 {
                gamma.data[i] = f32::sqrt(linear.data[i]);
            }
        }
        return gamma;
    }
}

impl Display for Vec3f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl From<f32> for Vec3f {
    fn from(value: f32) -> Self {
        return Self {
            data: [value, value, value],
        };
    }
}

impl From<[f32; 3]> for Vec3f {
    fn from(value: [f32; 3]) -> Self {
        return Self { data: value };
    }
}

impl From<[u8; 3]> for Vec3f {
    fn from(color: [u8; 3]) -> Self {
        return Vec3f::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
}

impl From<[u8; 4]> for Vec3f {
    fn from(color: [u8; 4]) -> Self {
        return Vec3f::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
}

impl From<Vec3f> for [u8; 3] {
    fn from(vector: Vec3f) -> Self {
        return [
            f32::floor(vector.x() * 255.0).clamp(0.0, 255.0) as u8,
            f32::floor(vector.y() * 255.0).clamp(0.0, 255.0) as u8,
            f32::floor(vector.z() * 255.0).clamp(0.0, 255.0) as u8,
        ];
    }
}

impl Add for Vec3f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self {
            data: [self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z()],
        };
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
        return Self {
            data: [self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z()],
        };
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
        return Self {
            data: [self.x() * rhs.x(), self.y() * rhs.y(), self.z() * rhs.z()],
        };
    }
}

impl Mul<f32> for Vec3f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        return Self {
            data: [self.x() * rhs, self.y() * rhs, self.z() * rhs],
        };
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
        return Self {
            data: [self.x() / rhs.x(), self.y() / rhs.y(), self.z() / rhs.z()],
        };
    }
}

impl Div<f32> for Vec3f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        return Self {
            data: [self.x() / rhs, self.y() / rhs, self.z() / rhs],
        };
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

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec2f {
    pub data: [f32; 2],
}

#[allow(dead_code)]
impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        return Self { data: [x, y] };
    }

    pub fn length(self) -> f32 {
        return f32::sqrt((self.x() * self.x()) + (self.y() * self.y()));
    }

    pub fn distance(a: Self, b: Self) -> f32 {
        return Self::sub(a, b).length();
    }

    pub fn normalized(self) -> Self {
        return Self {
            data: [self.x() / self.length(), self.y() / self.length()],
        };
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        return (a.x() * b.x()) + (a.y() * b.y());
    }

    pub fn min(a: Self, b: Self) -> Self {
        return Self {
            data: [f32::min(a.x(), b.x()), f32::min(a.y(), b.y())],
        };
    }

    pub fn max(a: Self, b: Self) -> Self {
        return Self {
            data: [f32::max(a.x(), b.x()), f32::max(a.y(), b.y())],
        };
    }

    pub fn abs(self) -> Self {
        return Self {
            data: [f32::abs(self.x()), f32::abs(self.y())],
        };
    }

    pub fn reversed(self) -> Self {
        return Self {
            data: [-self.x(), -self.y()],
        };
    }

    pub fn lerp(a: Self, b: Self, amount: f32) -> Self {
        return (a * (1.0 - amount)) + b * amount;
    }
}

impl Display for Vec2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl From<f32> for Vec2f {
    fn from(value: f32) -> Self {
        return Self {
            data: [value, value],
        };
    }
}

impl From<[f32; 2]> for Vec2f {
    fn from(value: [f32; 2]) -> Self {
        return Self { data: value };
    }
}

impl Add for Vec2f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self {
            data: [self.x() + rhs.x(), self.y() + rhs.y()],
        };
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
        return Self {
            data: [self.x() - rhs.x(), self.y() - rhs.y()],
        };
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
        return Self {
            data: [self.x() * rhs.x(), self.y() * rhs.y()],
        };
    }
}

impl Mul<f32> for Vec2f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        return Self {
            data: [self.x() * rhs, self.y() * rhs],
        };
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
        return Self {
            data: [self.x() / rhs.x(), self.y() / rhs.y()],
        };
    }
}

impl Div<f32> for Vec2f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        return Self {
            data: [self.x() / rhs, self.y() / rhs],
        };
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

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Mat4f {
    pub data: [[f32; 4]; 4],
}

impl Mat4f {
    pub fn new() -> Self {
        return Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
    }

    pub fn look_at(from: Vec3f, to: Vec3f, up: Vec3f) -> Self {
        let forward = (from - to).normalized();
        let right = Vec3f::cross(up, forward).normalized();
        let up = Vec3f::cross(forward, right);

        let mut m = Self::new();
        m.data[0][0] = right.x();
        m.data[0][1] = right.y();
        m.data[0][2] = right.z();
        m.data[1][0] = up.x();
        m.data[1][1] = up.y();
        m.data[1][2] = up.z();
        m.data[2][0] = forward.x();
        m.data[2][1] = forward.y();
        m.data[2][2] = forward.z();
        m.data[3][0] = from.x();
        m.data[3][1] = from.y();
        m.data[3][2] = from.z();
        return m;
    }

    // https://webgpufundamentals.org/webgpu/lessons/webgpu-cameras.html
    pub fn inverse(m: Self) -> Self {
        let mut dst = Mat4f::default();

        let m00 = m.data[0][0];
        let m01 = m.data[0][1];
        let m02 = m.data[0][2];
        let m03 = m.data[0][3];
        let m10 = m.data[1][0];
        let m11 = m.data[1][1];
        let m12 = m.data[1][2];
        let m13 = m.data[1][3];
        let m20 = m.data[2][0];
        let m21 = m.data[2][1];
        let m22 = m.data[2][2];
        let m23 = m.data[2][3];
        let m30 = m.data[3][0];
        let m31 = m.data[3][1];
        let m32 = m.data[3][2];
        let m33 = m.data[3][3];

        let tmp0 = m22 * m33;
        let tmp1 = m32 * m23;
        let tmp2 = m12 * m33;
        let tmp3 = m32 * m13;
        let tmp4 = m12 * m23;
        let tmp5 = m22 * m13;
        let tmp6 = m02 * m33;
        let tmp7 = m32 * m03;
        let tmp8 = m02 * m23;
        let tmp9 = m22 * m03;
        let tmp10 = m02 * m13;
        let tmp11 = m12 * m03;
        let tmp12 = m20 * m31;
        let tmp13 = m30 * m21;
        let tmp14 = m10 * m31;
        let tmp15 = m30 * m11;
        let tmp16 = m10 * m21;
        let tmp17 = m20 * m11;
        let tmp18 = m00 * m31;
        let tmp19 = m30 * m01;
        let tmp20 = m00 * m21;
        let tmp21 = m20 * m01;
        let tmp22 = m00 * m11;
        let tmp23 = m10 * m01;

        let t0 = (tmp0 * m11 + tmp3 * m21 + tmp4 * m31) - (tmp1 * m11 + tmp2 * m21 + tmp5 * m31);
        let t1 = (tmp1 * m01 + tmp6 * m21 + tmp9 * m31) - (tmp0 * m01 + tmp7 * m21 + tmp8 * m31);
        let t2 = (tmp2 * m01 + tmp7 * m11 + tmp10 * m31) - (tmp3 * m01 + tmp6 * m11 + tmp11 * m31);
        let t3 = (tmp5 * m01 + tmp8 * m11 + tmp11 * m21) - (tmp4 * m01 + tmp9 * m11 + tmp10 * m21);

        let d = 1.0 / (m00 * t0 + m10 * t1 + m20 * t2 + m30 * t3);

        dst.data[0][0] = d * t0;
        dst.data[0][1] = d * t1;
        dst.data[0][2] = d * t2;
        dst.data[0][3] = d * t3;

        dst.data[1][0] =
            d * ((tmp1 * m10 + tmp2 * m20 + tmp5 * m30) - (tmp0 * m10 + tmp3 * m20 + tmp4 * m30));
        dst.data[1][1] =
            d * ((tmp0 * m00 + tmp7 * m20 + tmp8 * m30) - (tmp1 * m00 + tmp6 * m20 + tmp9 * m30));
        dst.data[1][2] =
            d * ((tmp3 * m00 + tmp6 * m10 + tmp11 * m30) - (tmp2 * m00 + tmp7 * m10 + tmp10 * m30));
        dst.data[1][3] =
            d * ((tmp4 * m00 + tmp9 * m10 + tmp10 * m20) - (tmp5 * m00 + tmp8 * m10 + tmp11 * m20));

        dst.data[2][0] = d
            * ((tmp12 * m13 + tmp15 * m23 + tmp16 * m33)
                - (tmp13 * m13 + tmp14 * m23 + tmp17 * m33));
        dst.data[2][1] = d
            * ((tmp13 * m03 + tmp18 * m23 + tmp21 * m33)
                - (tmp12 * m03 + tmp19 * m23 + tmp20 * m33));
        dst.data[2][2] = d
            * ((tmp14 * m03 + tmp19 * m13 + tmp22 * m33)
                - (tmp15 * m03 + tmp18 * m13 + tmp23 * m33));
        dst.data[2][3] = d
            * ((tmp17 * m03 + tmp20 * m13 + tmp23 * m23)
                - (tmp16 * m03 + tmp21 * m13 + tmp22 * m23));

        dst.data[3][0] = d
            * ((tmp14 * m22 + tmp17 * m32 + tmp13 * m12)
                - (tmp16 * m32 + tmp12 * m12 + tmp15 * m22));
        dst.data[3][1] = d
            * ((tmp20 * m32 + tmp12 * m02 + tmp19 * m22)
                - (tmp18 * m22 + tmp21 * m32 + tmp13 * m02));
        dst.data[3][2] = d
            * ((tmp18 * m12 + tmp23 * m32 + tmp15 * m02)
                - (tmp22 * m32 + tmp14 * m02 + tmp19 * m12));
        dst.data[3][3] = d
            * ((tmp22 * m22 + tmp16 * m02 + tmp21 * m12)
                - (tmp20 * m12 + tmp23 * m22 + tmp17 * m02));

        return dst;
    }
}

impl Mul<Vec3f> for Mat4f {
    type Output = Vec3f;

    fn mul(self, rhs: Vec3f) -> Self::Output {
        let x = self.data[0][0] * rhs.x() + self.data[1][0] * rhs.y() + self.data[2][0] * rhs.z();
        let y = self.data[0][1] * rhs.x() + self.data[1][1] * rhs.y() + self.data[2][1] * rhs.z();
        let z = self.data[0][2] * rhs.x() + self.data[1][2] * rhs.y() + self.data[2][2] * rhs.z();
        return Vec3f::new(x, y, z);
    }
}
