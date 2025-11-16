use crate::Vec3f;
use crate::bvh::Node;
use crate::scene::{Scene, Triangle};
use crate::vector::Vec3Swizzles;

// NOTE: This is used when moving the ray towards a new direction on bounce AND for increasing node
// bounds by a tiny amount when checking intersections with rays to prevent visual holes in models
const RAY_HIT_OFFSET: f32 = 0.0001;

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3f,
    pub direction: Vec3f,
}

impl Ray {
    pub fn new(origin: Vec3f, direction: Vec3f) -> Self {
        return Self { origin, direction };
    }

    fn intersect_tri(ray: &Self, tri: &Triangle) -> HitInfo {
        let v_1 = Vec3f::from(tri.vertices[0].position);
        let v_2 = Vec3f::from(tri.vertices[1].position);
        let v_3 = Vec3f::from(tri.vertices[2].position);

        let edge_1 = v_2 - v_1;
        let edge_2 = v_3 - v_1;

        let ray_cross_e2 = Vec3f::cross(ray.direction, edge_2);
        let det = Vec3f::dot(edge_1, ray_cross_e2);

        let inv_det = 1.0 / det;
        let s = ray.origin - v_1;
        let u = inv_det * Vec3f::dot(s, ray_cross_e2);

        let s_cross_e1 = Vec3f::cross(s, edge_1);
        let v = inv_det * Vec3f::dot(ray.direction, s_cross_e1);

        let t = inv_det * Vec3f::dot(edge_2, s_cross_e1);

        let front_face = det > 0.0;

        let hit_point = ray.origin + (ray.direction * t);

        // Smooth shading
        let n_0: Vec3f = tri.vertices[0].normal.into();
        let n_1: Vec3f = tri.vertices[1].normal.into();
        let n_2: Vec3f = tri.vertices[2].normal.into();
        let mut normal: Vec3f = n_0 * (1.0 - u - v) + (n_1 * u) + (n_2 * v);
        if !front_face {
            normal = normal.reversed();
        }

        let t_0: Vec3f = Vec3f::new(
            tri.vertices[0].tex_coord[0],
            tri.vertices[0].tex_coord[1],
            0.0,
        );
        let t_1: Vec3f = Vec3f::new(
            tri.vertices[1].tex_coord[0],
            tri.vertices[1].tex_coord[1],
            0.0,
        );
        let t_2: Vec3f = Vec3f::new(
            tri.vertices[2].tex_coord[0],
            tri.vertices[2].tex_coord[1],
            0.0,
        );
        let uv = (t_0 * (1.0 - u - v) + (t_1 * u) + (t_2 * v)).xy();

        return HitInfo {
            has_hit: t > 0.0001
                && !(det < 0.0 && det > -0.0)
                && !(u < 0.0 || u > 1.0)
                && !(v < 0.0 || u + v > 1.0),
            point: hit_point,
            normal: normal,
            distance: t,
            uv: uv,
            material_id: tri.material_id,
            front_face: front_face,
        };
    }

    fn intersect_node(ray: &Self, node: &Node) -> bool {
        let t_min = (node.bounds_min - ray.origin) / ray.direction;
        let t_max = (node.bounds_max - ray.origin) / ray.direction;
        // NOTE: Adding and subtracting tiny amounts from t_1 and t_2 feels very hacky and not good
        let t_1 = Vec3f::min(t_min, t_max) - Vec3f::from(RAY_HIT_OFFSET);
        let t_2 = Vec3f::max(t_min, t_max) + Vec3f::from(RAY_HIT_OFFSET);
        let t_near = f32::max(f32::max(t_1.x(), t_1.y()), t_1.z());
        let t_far = f32::min(f32::min(t_2.x(), t_2.y()), t_2.z());
        return t_near < t_far && t_far > 0.0;
    }

    fn traverse_bvh(ray: &Self, scene: &Scene, index: usize, hit_info: &mut HitInfo) {
        let node = scene.bvh.nodes[index];
        if !Self::intersect_node(ray, &node) {
            return;
        }

        if node.num_tris > 0 {
            for i in 0..node.num_tris {
                let temp_hit_info = Self::intersect_tri(ray, &scene.tris[node.first_tri_id + i]);
                if temp_hit_info.has_hit && temp_hit_info.distance < hit_info.distance {
                    *hit_info = temp_hit_info;
                }
            }
        } else {
            Self::traverse_bvh(ray, scene, node.children_id, hit_info);
            Self::traverse_bvh(ray, scene, node.children_id + 1, hit_info);
        }
    }

    fn debug_bvh(ray: &Self, scene: &Scene, index: usize, debug_color: &mut Vec3f) {
        let node = scene.bvh.nodes[index];
        if !Self::intersect_node(ray, &node) {
            return;
        }

        if node.num_tris > 0 {
            if node.num_tris > 4 {
                *debug_color += Vec3f::new(0.05, 0.0, 0.0);
            } else {
                *debug_color += Vec3f::new(0.0, 0.05, 0.0);
            }
        } else {
            *debug_color += Vec3f::new(0.0, 0.0, 0.005);
            Self::debug_bvh(ray, scene, node.children_id, debug_color);
            Self::debug_bvh(ray, scene, node.children_id + 1, debug_color);
        }
    }

    fn schlick_fresnel(n_dot_v: f32, ior: f32) -> f32 {
        let f_0 = f32::powi(ior - 1.0, 2) / f32::powi(ior + 1.0, 2);
        return f_0 + (1.0 - f_0) * f32::powi(1.0 - n_dot_v, 5);
    }

    pub fn trace(
        ray: &mut Self,
        max_bounces: usize,
        scene: &Scene,
        rng_state: &mut u32,
        debug_bvh: bool,
    ) -> Vec3f {
        let mut ray_color = Vec3f::new(1.0, 1.0, 1.0);
        let mut incoming_light = Vec3f::new(0.0, 0.0, 0.0);
        let mut emitted_light = Vec3f::new(0.0, 0.0, 0.0);

        let mut curr_bounces: usize = 0;
        while curr_bounces < max_bounces {
            let mut hit_info = HitInfo::default();

            // Early return here because BVH visualization doesn't need more than one bounce
            if debug_bvh {
                Self::debug_bvh(ray, scene, 0, &mut incoming_light);
                return incoming_light;
            } else {
                Self::traverse_bvh(ray, scene, 0, &mut hit_info);
            }

            if hit_info.has_hit {
                let hit_material = &scene.materials[hit_info.material_id];
                let ior: f32;
                if hit_info.front_face {
                    ior = 1.0 / hit_material.ior;
                } else {
                    ior = hit_material.ior;
                }

                // Lambertian diffuse
                let diffuse =
                    (hit_info.normal + Vec3f::rand_in_unit_sphere(rng_state)).normalized();
                let new_dir = diffuse;

                *ray = Self::new(hit_info.point + new_dir * RAY_HIT_OFFSET, new_dir);

                if hit_material.base_color_tex_id != -1 {
                    ray_color *= Vec3f::from(
                        scene.textures[hit_material.base_color_tex_id as usize]
                            .color_at(hit_info.uv),
                    );
                } else {
                    ray_color *= hit_material.base_color;
                }
                if hit_material.emission_tex_id != -1 {
                    emitted_light += Vec3f::from(
                        scene.textures[hit_material.emission_tex_id as usize].color_at(hit_info.uv),
                    );
                } else {
                    emitted_light += hit_material.emission;
                }
                incoming_light += emitted_light * ray_color;

                curr_bounces += 1;
            } else {
                let sky_color = Vec3f::new(1.0, 1.0, 1.0);
                let sky_strength = Vec3f::from(1.0);

                ray_color *= sky_color;
                emitted_light += sky_strength;
                incoming_light += emitted_light * ray_color;

                break;
            }
        }

        // If we hit the sky directly
        if curr_bounces == 0 {
            return incoming_light;
        } else {
            return incoming_light / curr_bounces as f32;
        }
    }
}

struct HitInfo {
    has_hit: bool,
    point: Vec3f,
    normal: Vec3f,
    distance: f32,
    uv: [f32; 2],
    material_id: usize,
    front_face: bool,
}

impl Default for HitInfo {
    fn default() -> Self {
        return Self {
            has_hit: false,
            point: Vec3f::default(),
            normal: Vec3f::default(),
            distance: f32::MAX,
            uv: [0.0; 2],
            material_id: 0,
            front_face: false,
        };
    }
}
