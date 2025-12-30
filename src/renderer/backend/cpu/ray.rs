use crate::Vec3f;
use crate::bvh::Node;
use crate::scene::{Scene, Triangle};
use crate::vector::Vec3Swizzles;

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
            has_hit: t > 0.0
                && !(det < 0.0 && det > -0.0)
                && !(u < 0.0 || u > 1.0)
                && !(v < 0.0 || u + v > 1.0),
            point: ray.origin + (ray.direction * t),
            normal: normal,
            distance: t,
            uv: uv,
            material_id: tri.material_id,
            front_face: front_face,
        };
    }

    fn intersect_node(ray: &Self, node: &Node) -> f32 {
        let t_min = (node.bounds_min - ray.origin) / ray.direction;
        let t_max = (node.bounds_max - ray.origin) / ray.direction;
        let t_1 = Vec3f::min(t_min, t_max);
        let t_2 = Vec3f::max(t_min, t_max);
        let t_near = f32::max(f32::max(t_1.x(), t_1.y()), t_1.z());
        let t_far = f32::min(f32::min(t_2.x(), t_2.y()), t_2.z());
        if t_near <= t_far && t_far > 0.0 {
            return t_near;
        } else {
            return 1e30f32;
        }
    }

    // https://jacco.ompf2.com/2022/04/18/how-to-build-a-bvh-part-2-faster-rays/
    fn traverse_bvh(ray: &Self, scene: &Scene, hit_info: &mut HitInfo) {
        let mut stack: [Node; 64] = [Node::default(); 64];
        let mut node: &Node = scene.bvh.nodes.get(0).unwrap();
        let mut stack_ptr: usize = 0;

        loop {
            if node.num_tris > 0 {
                for i in 0..node.num_tris {
                    let temp_hit_info = Self::intersect_tri(
                        ray,
                        &scene.tris[(node.first_tri_or_child + i) as usize],
                    );
                    if temp_hit_info.has_hit && temp_hit_info.distance < hit_info.distance {
                        *hit_info = temp_hit_info;
                    }
                }
                if stack_ptr == 0 {
                    break;
                } else {
                    stack_ptr -= 1;
                    node = stack.get(stack_ptr).unwrap();
                }
                continue;
            }
            let mut child_1 = scene
                .bvh
                .nodes
                .get(node.first_tri_or_child as usize)
                .unwrap();
            let mut child_2 = scene
                .bvh
                .nodes
                .get((node.first_tri_or_child + 1) as usize)
                .unwrap();
            let mut dist_1 = Self::intersect_node(ray, &child_1);
            let mut dist_2 = Self::intersect_node(ray, &child_2);
            if dist_1 > dist_2 {
                std::mem::swap(&mut dist_1, &mut dist_2);
                std::mem::swap(&mut child_1, &mut child_2);
            }
            if dist_1 == 1e30f32 {
                if stack_ptr == 0 {
                    break;
                } else {
                    stack_ptr -= 1;
                    node = stack.get(stack_ptr).unwrap();
                }
            } else {
                node = &child_1;
                if dist_2 < 1e30f32 {
                    stack[stack_ptr as usize] = *child_2;
                    stack_ptr += 1;
                }
            }
        }
    }

    pub fn trace(ray: &mut Self, max_bounces: usize, scene: &Scene, rng_state: &mut u32) -> Vec3f {
        let mut ray_color = Vec3f::new(1.0, 1.0, 1.0);
        let mut incoming_light = Vec3f::new(0.0, 0.0, 0.0);
        let mut emitted_light = Vec3f::new(0.0, 0.0, 0.0);

        let mut curr_bounces: usize = 0;
        while curr_bounces < max_bounces {
            let mut hit_info = HitInfo::default();

            Self::traverse_bvh(ray, scene, &mut hit_info);

            if hit_info.has_hit {
                let hit_material = &scene.materials[hit_info.material_id as usize];
                let ior: f32;
                if hit_info.front_face {
                    ior = 1.0 / hit_material.ior;
                } else {
                    ior = hit_material.ior;
                }

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

                let new_dir =
                    (hit_info.normal + Vec3f::rand_in_unit_sphere(rng_state)).normalized();
                *ray = Self::new(hit_info.point + new_dir * 0.0001, new_dir);

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
    material_id: u32,
    front_face: bool,
}

impl Default for HitInfo {
    fn default() -> Self {
        return Self {
            has_hit: false,
            point: Vec3f::default(),
            normal: Vec3f::default(),
            distance: 1e30f32,
            uv: [0.0; 2],
            material_id: 0,
            front_face: false,
        };
    }
}
