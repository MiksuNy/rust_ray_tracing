use std::io::Write;

mod obj;

fn main() {
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.ppm")
        .unwrap();

    let width: i32 = 320;
    let height: i32 = 240;
    let aspect: f32 = width as f32 / height as f32;
    let sample_count: i32 = 1;
    let max_bounces: usize = 2;

    let _ = output_file.write(b"P3\n320 240\n255\n");

    let model = obj::load("../res/dragon.obj");

    let start_time = std::time::Instant::now();

    for y in (0..height).rev() {
        for x in 0..width {
            let mut final_color = Vec3::new(0.0, 0.0, 0.0);

            for curr_sample in 0..sample_count {
                let mut ray = Ray::new(
                    Vec3::new(0.0, 0.0, 0.7),
                    Vec3::new(
                        ((((x as f32 / width as f32) * 2.0) - 1.0) * aspect)
                            + rand(Vec3::new(
                                x as f32 * 8721.0,
                                curr_sample as f32 * 78612.0,
                                0.0,
                            )) * 0.001,
                        (((y as f32 / height as f32) * 2.0) - 1.0)
                            + rand(Vec3::new(
                                y as f32 * 4647.0,
                                curr_sample as f32 * 87124.0,
                                0.0,
                            )) * 0.001,
                        -1.0,
                    )
                    .normalized(),
                );

                final_color = Vec3::add(
                    final_color,
                    trace_ray(&mut ray, max_bounces, curr_sample, &model),
                );
            }

            final_color = Vec3::div(
                final_color,
                Vec3::new(
                    sample_count as f32,
                    sample_count as f32,
                    sample_count as f32,
                ),
            );

            final_color = linear_to_gamma(final_color);

            for c in final_color.to_color() {
                let _ = output_file.write(c.to_string().as_str().as_bytes());
                let _ = output_file.write(b" ");
            }
        }
        let _ = output_file.write(b"\n");

        println!("Lines remaining: {}", y);
    }

    let end_time = std::time::Instant::now();

    println!("Rendering took {} ms", (end_time - start_time).as_millis());
}

fn rand(seed: Vec3) -> f32 {
    return f32::fract(f32::sin(Vec3::dot(seed, Vec3::new(12.9898, 78.233, 0.0))));
}

fn linear_to_gamma(linear: Vec3) -> Vec3 {
    let mut gamma = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        if linear.data[i] > 0.0 {
            gamma.data[i] = f32::sqrt(linear.data[i]);
        }
    }
    return gamma;
}

fn trace_ray(ray: &mut Ray, max_bounces: usize, curr_sample: i32, model: &obj::OBJModel) -> Vec3 {
    let mut ray_color = Vec3::new(1.0, 1.0, 1.0);

    let mut curr_bounces = 0usize;
    while curr_bounces < max_bounces {
        let mut hit_info = HitInfo {
            hit_distance: 10000.0,
            hit_point: Vec3::new(0.0, 0.0, 0.0),
            hit_normal: Vec3::new(0.0, 0.0, 0.0),
            has_hit: false,
        };

        let mut i = 0usize;
        while i < model.vertex_buffer.len() - 2 {
            let tri = Triangle::new(
                Vec3::new(
                    model.vertex_buffer[i + 0].position[0],
                    model.vertex_buffer[i + 0].position[1],
                    model.vertex_buffer[i + 0].position[2],
                ),
                Vec3::new(
                    model.vertex_buffer[i + 1].position[0],
                    model.vertex_buffer[i + 1].position[1],
                    model.vertex_buffer[i + 1].position[2],
                ),
                Vec3::new(
                    model.vertex_buffer[i + 2].position[0],
                    model.vertex_buffer[i + 2].position[1],
                    model.vertex_buffer[i + 2].position[2],
                ),
            );

            let temp_hit_info = Ray::intersect_tri(*ray, tri);

            if temp_hit_info.has_hit && temp_hit_info.hit_distance < hit_info.hit_distance {
                hit_info = temp_hit_info;
            }

            i += 3;
        }

        //let rand_in_sphere = Vec3::new(
        //    rand(Vec3::mul_by_f32(
        //        hit_info.hit_point,
        //        8272193.0 + curr_sample as f32 * 73164.0,
        //    )),
        //    rand(Vec3::mul_by_f32(
        //        hit_info.hit_point,
        //        9826365.0 + curr_sample as f32 * 1876134.0,
        //    )),
        //    rand(Vec3::mul_by_f32(
        //        hit_info.hit_point,
        //        8731234.0 + curr_sample as f32 * 986134.0,
        //    )),
        //);
        //let rand_in_hemisphere = || -> Vec3 {
        //    if Vec3::dot(rand_in_sphere, hit_info.hit_normal) < 0.0 {
        //        return Vec3::new(
        //            -rand_in_sphere.data[0],
        //            -rand_in_sphere.data[1],
        //            -rand_in_sphere.data[2],
        //        );
        //    } else {
        //        return rand_in_sphere;
        //    }
        //};

        if hit_info.has_hit {
            let new_dir = Vec3::reflect(ray.direction, hit_info.hit_normal).normalized();
            *ray = Ray::new(
                Vec3::add(hit_info.hit_point, Vec3::mul_by_f32(new_dir, 0.001)),
                new_dir,
            );

            ray_color = Vec3::mul(ray_color, Vec3::new(0.5, 0.5, 0.5));

            curr_bounces += 1;
        } else {
            ray_color = Vec3::mul(ray_color, Vec3::new(1.0, 1.0, 1.0));

            curr_bounces += 1;

            break;
        }
    }

    return Vec3::div(
        ray_color,
        Vec3::new(
            curr_bounces as f32,
            curr_bounces as f32,
            curr_bounces as f32,
        ),
    );
}

#[derive(Clone, Copy)]
struct Vec3 {
    data: [f32; 3],
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { data: [x, y, z] };
    }

    fn to_color(self) -> [u32; 3] {
        return [
            f32::floor(self.data[0] * 255.0).clamp(0.0, 255.0) as u32,
            f32::floor(self.data[1] * 255.0).clamp(0.0, 255.0) as u32,
            f32::floor(self.data[2] * 255.0).clamp(0.0, 255.0) as u32,
        ];
    }

    fn length(self) -> f32 {
        return f32::sqrt(
            (self.data[0] * self.data[0])
                + (self.data[1] * self.data[1])
                + (self.data[2] * self.data[2]),
        );
    }

    fn normalized(self) -> Self {
        return Self {
            data: [
                self.data[0] / self.length(),
                self.data[1] / self.length(),
                self.data[2] / self.length(),
            ],
        };
    }

    fn reflect(incident: Self, normal: Self) -> Self {
        return Vec3::sub(
            incident,
            Vec3::mul_by_f32(normal, 2.0 * Vec3::dot(incident, normal)),
        );
    }

    /// eta = ratio of indices of refraction
    fn refract(incident: Self, normal: Self, eta: f32) -> Self {
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

    fn dot(a: Self, b: Self) -> f32 {
        return (a.data[0] * b.data[0]) + (a.data[1] * b.data[1]) + (a.data[2] * b.data[2]);
    }

    fn cross(a: Self, b: Self) -> Self {
        return Self {
            data: [
                (a.data[1] * b.data[2]) - (a.data[2] * b.data[1]),
                (a.data[2] * b.data[0]) - (a.data[0] * b.data[2]),
                (a.data[0] * b.data[1]) - (a.data[1] * b.data[0]),
            ],
        };
    }

    fn add(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] + b.data[0],
                a.data[1] + b.data[1],
                a.data[2] + b.data[2],
            ],
        };
    }

    fn sub(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] - b.data[0],
                a.data[1] - b.data[1],
                a.data[2] - b.data[2],
            ],
        };
    }

    fn mul(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] * b.data[0],
                a.data[1] * b.data[1],
                a.data[2] * b.data[2],
            ],
        };
    }

    fn mul_by_f32(vector: Vec3, scalar: f32) -> Self {
        return Self {
            data: [
                vector.data[0] * scalar,
                vector.data[1] * scalar,
                vector.data[2] * scalar,
            ],
        };
    }

    fn div(a: Self, b: Self) -> Self {
        return Self {
            data: [
                a.data[0] / b.data[0],
                a.data[1] / b.data[1],
                a.data[2] / b.data[2],
            ],
        };
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Self {
        return Self { origin, direction };
    }

    fn intersect_tri(ray: Self, tri: Triangle) -> HitInfo {
        let edge1 = tri.vertices[1];
        let edge2 = tri.vertices[2];

        let ray_cross_e2 = Vec3::cross(ray.direction, edge2);
        let det = Vec3::dot(edge1, ray_cross_e2);

        let inv_det = 1.0 / det;
        let s = Vec3::sub(ray.origin, tri.vertices[0]);
        let u = inv_det * Vec3::dot(s, ray_cross_e2);

        let s_cross_e1 = Vec3::cross(s, edge1);
        let v = inv_det * Vec3::dot(ray.direction, s_cross_e1);

        let t = inv_det * Vec3::dot(edge2, s_cross_e1);

        return HitInfo {
            has_hit: t > 0.001
                && !(det < 0.0)
                && !(u < 0.0 || u > 1.0)
                && !(v < 0.0 || u + v > 1.0),
            hit_point: Vec3::add(ray.origin, Vec3::mul_by_f32(ray.direction, t)),
            hit_normal: Vec3::normalized(Vec3::cross(edge1, edge2)),
            hit_distance: t,
        };
    }
}

struct HitInfo {
    has_hit: bool,
    hit_point: Vec3,
    hit_normal: Vec3,
    hit_distance: f32,
}

#[derive(Clone, Copy)]
struct Triangle {
    vertices: [Vec3; 3],
}

impl Triangle {
    fn new(p1: Vec3, e1: Vec3, e2: Vec3) -> Self {
        return Self {
            vertices: [p1, e1, e2],
        };
    }
}
