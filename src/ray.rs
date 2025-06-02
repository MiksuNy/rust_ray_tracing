use crate::Vec3;
use crate::obj::Model;
use crate::obj::Triangle;

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

        let front_face = det > 0.0;

        let normal: Vec3;
        if front_face {
            normal = Vec3::normalized(Vec3::cross(edge1, edge2));
        } else {
            normal = Vec3::normalized(Vec3::cross(edge2, edge1));
        }

        return HitInfo {
            has_hit: t > 0.001
                && !(det < 0.0)
                && !(u < 0.0 || u > 1.0)
                && !(v < 0.0 || u + v > 1.0),
            hit_point: Vec3::add(ray.origin, Vec3::mul_by_f32(ray.direction, t)),
            hit_normal: normal,
            hit_distance: t,
            hit_material_id: tri.material_id,
        };
    }

    pub fn trace(mut ray: Self, max_bounces: usize, model: Model, rng_state: &mut u32) -> Vec3 {
        let mut ray_color = Vec3::new(1.0, 1.0, 1.0);
        let mut incoming_light = Vec3::new(0.0, 0.0, 0.0);
        let mut emitted_light = Vec3::new(0.0, 0.0, 0.0);

        let mut curr_bounces = 0usize;
        while curr_bounces < max_bounces {
            let mut hit_info = HitInfo {
                hit_distance: 100000.0,
                hit_point: Vec3::new(0.0, 0.0, 0.0),
                hit_normal: Vec3::new(0.0, 0.0, 0.0),
                has_hit: false,
                hit_material_id: 0,
            };

            for tri in model.tris.clone() {
                let temp_hit_info = Ray::intersect_tri(ray, tri);
                if temp_hit_info.has_hit && temp_hit_info.hit_distance < hit_info.hit_distance {
                    hit_info = temp_hit_info;
                }
            }

            if hit_info.has_hit {
                let hit_material = model
                    .materials
                    .iter()
                    .nth(hit_info.hit_material_id)
                    .unwrap();

                let new_dir: Vec3;
                let unit_sphere = Vec3::rand_in_unit_sphere(rng_state);
                if Vec3::dot(unit_sphere, hit_info.hit_normal) > 0.0 {
                    new_dir = unit_sphere;
                } else {
                    new_dir = unit_sphere.reverse();
                }
                ray = Ray::new(hit_info.hit_point, new_dir);

                emitted_light = Vec3::add(emitted_light, hit_material.emission);
                ray_color = Vec3::mul(ray_color, hit_material.diffuse_color);
                incoming_light = Vec3::add(emitted_light, ray_color);

                curr_bounces += 1;
            } else {
                let sky_color = Vec3::new(0.0, 0.0, 0.0);
                emitted_light = Vec3::add(emitted_light, sky_color);
                ray_color = Vec3::mul(ray_color, sky_color);
                incoming_light = Vec3::add(emitted_light, ray_color);

                curr_bounces += 1;

                break;
            }
        }

        return Vec3::div(incoming_light, Vec3::from_f32(curr_bounces as f32));
    }
}

pub struct HitInfo {
    pub has_hit: bool,
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_distance: f32,
    pub hit_material_id: usize,
}
