use crate::Vec3;
use crate::bvh::Triangle;
use crate::obj::Model;

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
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

    pub fn trace(ray: &mut Self, max_bounces: usize, model: &Model, rng_state: &mut u32) -> Vec3 {
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
}

pub struct HitInfo {
    pub has_hit: bool,
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_distance: f32,
}
