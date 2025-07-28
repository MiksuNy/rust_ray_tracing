use std::fmt::Display;

#[derive(Clone, Copy, Default)]
pub struct Vec3 {
    pub data: [f32; 3],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { data: [x, y, z] };
    }

    pub fn from_array(data: [f32; 3]) -> Self {
        return Self { data };
    }

    pub fn from_f32(f: f32) -> Self {
        return Self { data: [f, f, f] };
    }

    pub fn to_color(self) -> [u32; 3] {
        return [
            f32::floor(self.data[0] * 255.0).clamp(0.0, 255.0) as u32,
            f32::floor(self.data[1] * 255.0).clamp(0.0, 255.0) as u32,
            f32::floor(self.data[2] * 255.0).clamp(0.0, 255.0) as u32,
        ];
    }

    pub fn length(self) -> f32 {
        return f32::sqrt(
            (self.data[0] * self.data[0])
                + (self.data[1] * self.data[1])
                + (self.data[2] * self.data[2]),
        );
    }

    pub fn normalized(self) -> Self {
        return Self {
            data: [
                self.data[0] / self.length(),
                self.data[1] / self.length(),
                self.data[2] / self.length(),
            ],
        };
    }

    pub fn reflect(incident: Self, normal: Self) -> Self {
        return Self::sub(
            incident,
            Self::mul_by_f32(normal, 2.0 * Self::dot(incident, normal)),
        );
    }

    /// eta = ratio of indices of refraction
    pub fn refract(incident: Self, normal: Self, eta: f32) -> Self {
        let k =
            1.0 - (eta * eta) * (1.0 - (Self::dot(normal, incident) * Self::dot(normal, incident)));
        if k < 0.0 {
            return Self::new(0.0, 0.0, 0.0);
        } else {
            let eta_dot_n_i = eta * Self::dot(normal, incident);
            return Self::sub(
                Self::mul_by_f32(incident, eta),
                Self::mul(
                    Self::new(
                        eta_dot_n_i + f32::sqrt(k),
                        eta_dot_n_i + f32::sqrt(k),
                        eta_dot_n_i + f32::sqrt(k),
                    ),
                    normal,
                ),
            );
        }
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        return (a.data[0] * b.data[0]) + (a.data[1] * b.data[1]) + (a.data[2] * b.data[2]);
    }

    pub fn cross(a: Self, b: Self) -> Self {
        return Self {
            data: [
                (a.data[1] * b.data[2]) - (a.data[2] * b.data[1]),
                (a.data[2] * b.data[0]) - (a.data[0] * b.data[2]),
                (a.data[0] * b.data[1]) - (a.data[1] * b.data[0]),
            ],
        };
    }

    pub fn add(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] + b.data[0],
                a.data[1] + b.data[1],
                a.data[2] + b.data[2],
            ],
        };
    }

    pub fn sub(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] - b.data[0],
                a.data[1] - b.data[1],
                a.data[2] - b.data[2],
            ],
        };
    }

    pub fn mul(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] * b.data[0],
                a.data[1] * b.data[1],
                a.data[2] * b.data[2],
            ],
        };
    }

    pub fn mul_by_f32(vector: Self, scalar: f32) -> Self {
        return Self {
            data: [
                vector.data[0] * scalar,
                vector.data[1] * scalar,
                vector.data[2] * scalar,
            ],
        };
    }

    pub fn div(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] / b.data[0],
                a.data[1] / b.data[1],
                a.data[2] / b.data[2],
            ],
        };
    }

    pub fn min(a: Self, b: Self) -> Self {
        return Self {
            data: [
                f32::min(a.data[0], b.data[0]),
                f32::min(a.data[1], b.data[1]),
                f32::min(a.data[2], b.data[2]),
            ],
        };
    }

    pub fn max(a: Self, b: Self) -> Self {
        return Self {
            data: [
                f32::max(a.data[0], b.data[0]),
                f32::max(a.data[1], b.data[1]),
                f32::max(a.data[2], b.data[2]),
            ],
        };
    }

    pub fn abs(self) -> Self {
        return Self {
            data: [
                f32::abs(self.data[0]),
                f32::abs(self.data[1]),
                f32::abs(self.data[2]),
            ],
        };
    }

    pub fn reversed(self) -> Self {
        return Self {
            data: [-self.data[0], -self.data[1], -self.data[2]],
        };
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

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.data[0], self.data[1], self.data[2])
    }
}
