#[derive(Clone, Copy)]
pub struct Vec3 {
    pub data: [f32; 3],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { data: [x, y, z] };
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
        return Vec3::sub(
            incident,
            Vec3::mul_by_f32(normal, 2.0 * Vec3::dot(incident, normal)),
        );
    }

    /// eta = ratio of indices of refraction
    pub fn refract(incident: Self, normal: Self, eta: f32) -> Self {
        let k =
            1.0 - (eta * eta) * (1.0 - (Vec3::dot(normal, incident) * Vec3::dot(normal, incident)));
        if k < 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        } else {
            let eta_dot_n_i = eta * Vec3::dot(normal, incident);
            return Vec3::sub(
                Vec3::mul_by_f32(incident, eta),
                Vec3::mul(
                    Vec3::new(
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

    pub fn reverse(&self) -> Self {
        return Self {
            data: [-self.data[0], -self.data[1], -self.data[2]],
        };
    }

    // https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
    fn pcg_hash(input: &mut u32) -> u32 {
        *input = input.wrapping_mul(747796405u32 + 2891336453u32);
        let word: u32 =
            ((*input >> ((*input >> 28u32) + 4u32)) ^ *input).wrapping_mul(277803737u32);
        return (word >> 22u32) ^ word;
    }

    pub fn rand_f32(input: &mut u32) -> f32 {
        return Vec3::pcg_hash(input) as f32 / u32::MAX as f32;
    }

    fn rand_f32_nd(input: &mut u32) -> f32 {
        let theta = 6.283185 * Vec3::rand_f32(input);
        let rho = f32::sqrt(-2.0 * f32::log10(Vec3::rand_f32(input)));
        return rho * f32::cos(theta);
    }

    pub fn rand_in_unit_sphere(input: &mut u32) -> Self {
        return Self {
            data: [
                (Vec3::rand_f32_nd(input) * 2.0) - 1.0,
                (Vec3::rand_f32_nd(input) * 2.0) - 1.0,
                (Vec3::rand_f32_nd(input) * 2.0) - 1.0,
            ],
        }
        .normalized();
    }
}
