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

    pub fn mul_by_f32(vector: Vec3, scalar: f32) -> Self {
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
}
