use crate::Vec3;
use crate::bvh::Node;
use crate::obj::{Model, Triangle};

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

struct HitInfo {
    has_hit: bool,
    hit_point: Vec3,
    hit_normal: Vec3,
    hit_distance: f32,
    hit_material_id: usize,
}

impl Default for HitInfo {
    fn default() -> Self {
        return Self {
            has_hit: false,
            hit_point: Vec3::default(),
            hit_normal: Vec3::default(),
            hit_distance: f32::MAX,
            hit_material_id: 0,
        };
    }
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        return Self { origin, direction };
    }

    fn intersect_tri(ray: &Self, tri: &Triangle) -> HitInfo {
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
            has_hit: t > 0.0001
                && !(det < 0.0)
                && !(u < 0.0 || u > 1.0)
                && !(v < 0.0 || u + v > 1.0),
            hit_point: Vec3::add(ray.origin, Vec3::mul_by_f32(ray.direction, t)),
            hit_normal: normal,
            hit_distance: t,
            hit_material_id: tri.material_id,
        };
    }

    fn intersect_node(ray: &Self, node: &Node) -> bool {
        let t_min = Vec3::div(Vec3::sub(node.bounds_min, ray.origin), ray.direction);
        let t_max = Vec3::div(Vec3::sub(node.bounds_max, ray.origin), ray.direction);
        let t_1 = Vec3::min(t_min, t_max);
        let t_2 = Vec3::max(t_min, t_max);
        let t_near = f32::max(f32::max(t_1.data[0], t_1.data[1]), t_1.data[2]);
        let t_far = f32::min(f32::min(t_2.data[0], t_2.data[1]), t_2.data[2]);
        return t_near < t_far;
    }

    fn traverse_bvh(ray: &Self, model: &Model, index: usize, hit_info: &mut HitInfo) {
        let node = model.bvh.nodes[index];
        if !Self::intersect_node(ray, &node) {
            return;
        }

        if node.num_tris > 0 {
            for i in 0..node.num_tris {
                let temp_hit_info = Self::intersect_tri(ray, &model.tris[node.first_tri_id + i]);
                if temp_hit_info.has_hit && temp_hit_info.hit_distance < hit_info.hit_distance {
                    *hit_info = temp_hit_info;
                }
            }
        } else {
            Self::traverse_bvh(ray, model, node.children_id, hit_info);
            Self::traverse_bvh(ray, model, node.children_id + 1, hit_info);
        }
    }

    fn debug_bvh(ray: &Self, model: &Model, index: usize, debug_color: &mut Vec3) {
        let node = model.bvh.nodes[index];
        if !Self::intersect_node(ray, &node) {
            return;
        }

        if node.num_tris > 0 {
            *debug_color = Vec3::add(*debug_color, Vec3::new(0.05, 0.0, 0.0));
        } else {
            Self::debug_bvh(ray, model, node.children_id, debug_color);
            Self::debug_bvh(ray, model, node.children_id + 1, debug_color);
        }
    }

    pub fn trace(
        ray: &mut Self,
        max_bounces: usize,
        model: &Model,
        rng_state: &mut u32,
        debug_bvh: bool,
    ) -> Vec3 {
        let mut ray_color = Vec3::new(1.0, 1.0, 1.0);
        let mut incoming_light = Vec3::new(0.0, 0.0, 0.0);
        let mut emitted_light = Vec3::new(0.0, 0.0, 0.0);

        let mut curr_bounces: usize = 0;
        while curr_bounces < max_bounces {
            let mut hit_info = HitInfo::default();

            // Early return here because BVH visualization doesn't need more than one bounce
            if debug_bvh {
                Self::debug_bvh(ray, model, 0, &mut incoming_light);
                return incoming_light;
            } else {
                Self::traverse_bvh(ray, model, 0, &mut hit_info);
            }

            if hit_info.has_hit {
                let hit_material = &model.materials[hit_info.hit_material_id];

                let new_dir = Vec3::rand_in_unit_hemisphere(rng_state, hit_info.hit_normal);
                *ray = Self::new(
                    Vec3::add(hit_info.hit_point, Vec3::mul_by_f32(new_dir, 0.0001)),
                    new_dir,
                );

                emitted_light = Vec3::add(emitted_light, hit_material.emission);
                ray_color = Vec3::mul(ray_color, hit_material.diffuse_color);
                incoming_light = Vec3::add(incoming_light, Vec3::mul(emitted_light, ray_color));

                curr_bounces += 1;
            } else {
                let sky_color = Vec3::new(0.8, 0.8, 0.8);
                ray_color = Vec3::mul(ray_color, sky_color);
                incoming_light = Vec3::add(incoming_light, ray_color);

                curr_bounces += 1;

                break;
            }
        }

        return Vec3::div(incoming_light, Vec3::from_f32(curr_bounces as f32));
    }
}
