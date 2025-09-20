use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, Default)]
pub struct Vec3f {
    pub data: [f32; 3],
}

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
pub trait Vec3Swizzles<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;

    fn xx(&self) -> [T; 2];
    fn xy(&self) -> [T; 2];
    fn xz(&self) -> [T; 2];
    fn yx(&self) -> [T; 2];
    fn yy(&self) -> [T; 2];
    fn yz(&self) -> [T; 2];
    fn zx(&self) -> [T; 2];
    fn zy(&self) -> [T; 2];
    fn zz(&self) -> [T; 2];

    fn xxx(&self) -> [T; 3];
    fn xxy(&self) -> [T; 3];
    fn xxz(&self) -> [T; 3];
    fn yxx(&self) -> [T; 3];
    fn yxy(&self) -> [T; 3];
    fn yxz(&self) -> [T; 3];
    fn zxx(&self) -> [T; 3];
    fn zxy(&self) -> [T; 3];
    fn zxz(&self) -> [T; 3];
    fn xyx(&self) -> [T; 3];
    fn xyy(&self) -> [T; 3];
    fn xyz(&self) -> [T; 3];
    fn yyx(&self) -> [T; 3];
    fn yyy(&self) -> [T; 3];
    fn yyz(&self) -> [T; 3];
    fn zyx(&self) -> [T; 3];
    fn zyy(&self) -> [T; 3];
    fn zyz(&self) -> [T; 3];
    fn xzx(&self) -> [T; 3];
    fn xzy(&self) -> [T; 3];
    fn xzz(&self) -> [T; 3];
    fn yzx(&self) -> [T; 3];
    fn yzy(&self) -> [T; 3];
    fn yzz(&self) -> [T; 3];
    fn zzx(&self) -> [T; 3];
    fn zzy(&self) -> [T; 3];
    fn zzz(&self) -> [T; 3];
}

impl Vec3Swizzles<f32> for Vec3f {
    fn x(&self) -> f32 {
        return self.data[0];
    }

    fn y(&self) -> f32 {
        return self.data[1];
    }

    fn z(&self) -> f32 {
        return self.data[2];
    }

    fn xx(&self) -> [f32; 2] {
        return [self.data[0], self.data[0]];
    }

    fn xy(&self) -> [f32; 2] {
        return [self.data[0], self.data[1]];
    }

    fn xz(&self) -> [f32; 2] {
        return [self.data[0], self.data[2]];
    }

    fn yx(&self) -> [f32; 2] {
        return [self.data[1], self.data[0]];
    }

    fn yy(&self) -> [f32; 2] {
        return [self.data[1], self.data[1]];
    }

    fn yz(&self) -> [f32; 2] {
        return [self.data[1], self.data[2]];
    }

    fn zx(&self) -> [f32; 2] {
        return [self.data[2], self.data[0]];
    }

    fn zy(&self) -> [f32; 2] {
        return [self.data[2], self.data[1]];
    }

    fn zz(&self) -> [f32; 2] {
        return [self.data[2], self.data[2]];
    }

    fn xxx(&self) -> [f32; 3] {
        return [self.data[0], self.data[0], self.data[0]];
    }

    fn xxy(&self) -> [f32; 3] {
        return [self.data[0], self.data[0], self.data[1]];
    }

    fn xxz(&self) -> [f32; 3] {
        return [self.data[0], self.data[0], self.data[2]];
    }

    fn yxx(&self) -> [f32; 3] {
        return [self.data[1], self.data[0], self.data[0]];
    }

    fn yxy(&self) -> [f32; 3] {
        return [self.data[1], self.data[0], self.data[1]];
    }

    fn yxz(&self) -> [f32; 3] {
        return [self.data[1], self.data[0], self.data[2]];
    }

    fn zxx(&self) -> [f32; 3] {
        return [self.data[2], self.data[0], self.data[0]];
    }

    fn zxy(&self) -> [f32; 3] {
        return [self.data[2], self.data[0], self.data[1]];
    }

    fn zxz(&self) -> [f32; 3] {
        return [self.data[2], self.data[0], self.data[2]];
    }

    fn xyx(&self) -> [f32; 3] {
        return [self.data[0], self.data[1], self.data[0]];
    }

    fn xyy(&self) -> [f32; 3] {
        return [self.data[0], self.data[1], self.data[1]];
    }

    fn xyz(&self) -> [f32; 3] {
        return [self.data[0], self.data[1], self.data[2]];
    }

    fn yyx(&self) -> [f32; 3] {
        return [self.data[1], self.data[1], self.data[0]];
    }

    fn yyy(&self) -> [f32; 3] {
        return [self.data[1], self.data[1], self.data[1]];
    }

    fn yyz(&self) -> [f32; 3] {
        return [self.data[1], self.data[1], self.data[2]];
    }

    fn zyx(&self) -> [f32; 3] {
        return [self.data[2], self.data[1], self.data[0]];
    }

    fn zyy(&self) -> [f32; 3] {
        return [self.data[2], self.data[1], self.data[1]];
    }

    fn zyz(&self) -> [f32; 3] {
        return [self.data[2], self.data[1], self.data[2]];
    }

    fn xzx(&self) -> [f32; 3] {
        return [self.data[0], self.data[2], self.data[0]];
    }

    fn xzy(&self) -> [f32; 3] {
        return [self.data[0], self.data[2], self.data[1]];
    }

    fn xzz(&self) -> [f32; 3] {
        return [self.data[0], self.data[2], self.data[2]];
    }

    fn yzx(&self) -> [f32; 3] {
        return [self.data[1], self.data[2], self.data[0]];
    }

    fn yzy(&self) -> [f32; 3] {
        return [self.data[1], self.data[2], self.data[1]];
    }

    fn yzz(&self) -> [f32; 3] {
        return [self.data[1], self.data[2], self.data[2]];
    }

    fn zzx(&self) -> [f32; 3] {
        return [self.data[2], self.data[2], self.data[0]];
    }

    fn zzy(&self) -> [f32; 3] {
        return [self.data[2], self.data[2], self.data[1]];
    }

    fn zzz(&self) -> [f32; 3] {
        return [self.data[2], self.data[2], self.data[2]];
    }
}
