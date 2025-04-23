use crate::vec3::Vec3;
use std::io::Write;

mod obj;
mod vec3;

fn main() {
    let mut rng_state: u32;

    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.ppm")
        .unwrap();

    let width: u32 = 320;
    let height: u32 = 240;
    let aspect: f32 = width as f32 / height as f32;
    let sample_count: usize = 4;
    let max_bounces: usize = 4;

    let _ = output_file.write(b"P3\n320 240\n255\n");

    let model = obj::load("../res/cube_with_floor.obj");

    let start_time = std::time::Instant::now();

    for y in (0..height).rev() {
        for x in 0..width {
            let mut final_color = Vec3::new(0.0, 0.0, 0.0);

            for curr_sample in 0..sample_count {
                let ndc_x: f32 = x as f32 / width as f32;
                let ndc_y: f32 = y as f32 / height as f32;

                let screen_x: f32 = ((ndc_x * 2.0) - 1.0) * aspect;
                let screen_y: f32 = (ndc_y * 2.0) - 1.0;

                rng_state = ((ndc_x / width as f32 * 95729371.0 + curr_sample as f32)
                    + (ndc_y / height as f32 * 43879571.0 + curr_sample as f32))
                    as u32;

                let mut ray = Ray::new(
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(
                        screen_x + Vec3::rand_f32(&mut rng_state) * 0.001,
                        screen_y + Vec3::rand_f32(&mut rng_state) * 0.001,
                        -1.0,
                    )
                    .normalized(),
                );

                final_color = Vec3::add(
                    final_color,
                    trace_ray(&mut ray, max_bounces, &model, &mut rng_state),
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

fn linear_to_gamma(linear: Vec3) -> Vec3 {
    let mut gamma = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        if linear.data[i] > 0.0 {
            gamma.data[i] = f32::sqrt(linear.data[i]);
        }
    }
    return gamma;
}

fn trace_ray(ray: &mut Ray, max_bounces: usize, model: &obj::Model, rng_state: &mut u32) -> Vec3 {
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

        if hit_info.has_hit {
            let unit_sphere = Vec3::rand_in_unit_sphere(rng_state);
            let unit_hemisphere = match Vec3::dot(unit_sphere, hit_info.hit_normal) > 0.0 {
                true => unit_sphere,
                false => unit_sphere.reverse(),
            };

            *ray = Ray::new(
                Vec3::add(hit_info.hit_point, Vec3::mul_by_f32(unit_hemisphere, 0.001)),
                unit_hemisphere,
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
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Self {
        return Self { origin, direction };
    }

    fn intersect_tri(ray: Self, tri: Triangle) -> HitInfo {
        let edge1 = Vec3::sub(tri.vertices[1], tri.vertices[0]);
        let edge2 = Vec3::sub(tri.vertices[2], tri.vertices[0]);

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
    fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        return Self {
            vertices: [p1, p2, p3],
        };
    }
}
