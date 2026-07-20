use std::collections::HashMap;

use crate::bvh::AABB;
use crate::bvh::BVH;
use crate::loader::obj::OBJ;
use crate::log_error;
use crate::math::mat4::Mat4f;
use crate::math::vec::*;
use crate::math::vec3::*;
use crate::texture::Texture;

/// Representation of a 3D scene for use in the ray tracer.
#[derive(Clone, Default)]
pub struct Scene {
    pub tris: Vec<Triangle>,
    pub materials: HashMap<String, Material>,
    pub textures: Vec<Texture>,
    pub bvh: BVH,
    pub camera: Camera,
}

impl Scene {
    pub fn load(path: &str) -> Option<Self> {
        if !std::fs::exists(path).unwrap() {
            log_error!("Could not find scene at path: '{}'", path);
            return None;
        }

        let format = path.split(".").last().unwrap();
        match format {
            "obj" => Some(OBJ::load(path).into()),
            _ => {
                log_error!("Unsupported scene format '{}' at path '{}'", format, path);
                return None;
            }
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
        self.camera.update_view();
    }
}

impl From<OBJ> for Scene {
    fn from(obj: OBJ) -> Self {
        let mut scene = Scene::default();

        for obj_tri in obj.tris {
            let mut vertices: [Vertex; 3] = [Vertex::default(); 3];
            for i in 0..3 {
                let position = *obj
                    .vertex_buffer
                    .positions
                    .get(obj_tri.positions[i])
                    .unwrap_or(&[0.0; 3]);
                let tex_coord = *obj
                    .vertex_buffer
                    .tex_coords
                    .get(obj_tri.tex_coords[i])
                    .unwrap_or(&[0.0; 2]);
                let normal = *obj
                    .vertex_buffer
                    .normals
                    .get(obj_tri.normals[i])
                    .unwrap_or(&[0.0; 3]);
                vertices[i] = Vertex {
                    position: position.into(),
                    tex_coord_x: tex_coord[0],
                    normal: normal.into(),
                    tex_coord_y: tex_coord[1],
                };
            }
            scene
                .tris
                .push(Triangle::new(vertices, obj_tri.material_id));
        }

        scene.materials = obj.materials;
        scene.textures = obj.textures;

        BVH::build(&mut scene, false);

        return scene;
    }
}

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Vertex {
    pub position: Vec3f,
    pub tex_coord_x: f32,
    pub normal: Vec3f,
    pub tex_coord_y: f32,
}

// This needs to derive some bytemuck traits so we can put 'em in a buffer on the GPU
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub material_id: u32,
    _pad: [u8; 12],
}

impl Triangle {
    fn new(vertices: [Vertex; 3], material_id: u32) -> Self {
        return Self {
            vertices,
            material_id,
            _pad: [0; 12],
        };
    }

    pub fn bounds(&self) -> (Vec3f, Vec3f) {
        let mut bounds = (Vec3f::from(f32::MAX), Vec3f::from(f32::MIN));
        bounds.grow_by_tri(self);
        return (bounds.0, bounds.1);
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Material {
    pub base_color: Vec3f,
    pub transmission: f32,
    pub specular_tint: Vec3f,
    pub ior: f32,
    pub emission: Vec3f,
    pub roughness: f32,
    pub metallic: f32,
    pub transparency: f32,
    pub base_color_tex_id: u32,
    pub transparency_tex_id: u32,
    pub roughness_tex_id: u32,
    pub metallic_tex_id: u32,
    pub emission_tex_id: u32,
    pub normal_tex_id: u32,
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            base_color: Vec3f::from(0.8),
            transmission: 0.0,
            specular_tint: Vec3f::from(1.0),
            ior: 1.45,
            emission: Vec3f::new(0.0, 0.0, 0.0),
            roughness: 1.0,
            metallic: 0.0,
            transparency: 1.0,
            base_color_tex_id: u32::MAX,
            transparency_tex_id: u32::MAX,
            roughness_tex_id: u32::MAX,
            metallic_tex_id: u32::MAX,
            emission_tex_id: u32::MAX,
            normal_tex_id: u32::MAX,
        };
    }
}

#[derive(Clone, Default)]
pub struct Camera {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3f,
    pub forward: Vec3f,
    pub up: Vec3f,
    pub right: Vec3f,
    pub look_at: Mat4f,
}

impl Camera {
    pub fn update_view(&mut self) {
        let direction = Vec3f::new(
            f32::cos(f32::to_radians(self.yaw)) * f32::cos(f32::to_radians(self.pitch)),
            f32::sin(f32::to_radians(self.pitch)),
            f32::sin(f32::to_radians(self.yaw)) * f32::cos(f32::to_radians(self.pitch)),
        );
        let world_up = Vec3f::new(0.0, 1.0, 0.0);

        self.forward = direction.normalized();
        self.right = Vec3f::cross(world_up, self.forward).normalized();
        self.up = Vec3f::cross(self.forward, self.right);

        self.look_at = Mat4f::look_at(self.position, self.position + self.forward, self.up);
    }
}
