use crate::bvh::BVH;
use crate::loader::obj::OBJ;
use crate::log_error;
use crate::texture::Texture;
use crate::vector::{Mat4f, Vec3f};

/// Representation of a 3D scene for use in the ray tracer.
#[derive(Clone, Default)]
pub struct Scene {
    pub tris: Vec<Triangle>,
    pub materials: Vec<Material>,
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

    pub fn set_camera(&mut self, camera: Camera) -> &Self {
        self.camera = camera;
        return self;
    }
}

impl From<OBJ> for Scene {
    fn from(obj: OBJ) -> Self {
        let mut scene = Scene::default();

        for obj_tri in obj.tris {
            let mut vertices: [Vertex; 3] = [Vertex::default(); 3];
            for i in 0..3 {
                vertices[i] = Vertex {
                    position: *obj
                        .vertex_buffer
                        .positions
                        .get(obj_tri.positions[i])
                        .unwrap_or(&[0.0; 3]),
                    _pad1: [0; 4],
                    normal: *obj
                        .vertex_buffer
                        .normals
                        .get(obj_tri.normals[i])
                        .unwrap_or(&[0.0; 3]),
                    _pad2: [0; 4],
                    tex_coord: *obj
                        .vertex_buffer
                        .tex_coords
                        .get(obj_tri.tex_coords[i])
                        .unwrap_or(&[0.0; 2]),
                    _pad3: [0; 8],
                };
            }
            scene
                .tris
                .push(Triangle::new(vertices, obj_tri.material_id));
        }

        scene.materials = obj.materials;
        scene.textures = obj.textures;

        BVH::build(&mut scene);

        return scene;
    }
}

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Vertex {
    pub position: [f32; 3],
    _pad1: [u8; 4],
    pub normal: [f32; 3],
    _pad2: [u8; 4],
    pub tex_coord: [f32; 2],
    _pad3: [u8; 8],
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

    pub fn mid(&self) -> Vec3f {
        return Vec3f::new(
            (self.vertices[0].position[0]
                + self.vertices[1].position[0]
                + self.vertices[2].position[0])
                / 3.0,
            (self.vertices[0].position[1]
                + self.vertices[1].position[1]
                + self.vertices[2].position[1])
                / 3.0,
            (self.vertices[0].position[2]
                + self.vertices[1].position[2]
                + self.vertices[2].position[2])
                / 3.0,
        );
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Material {
    pub base_color: Vec3f,
    _pad1: [u8; 4],
    pub specular_tint: Vec3f,
    _pad2: [u8; 4],
    pub emission: Vec3f,
    pub transmission: f32,
    pub ior: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub base_color_tex_id: u32,
    pub emission_tex_id: u32,
    _pad3: [u8; 12],
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            base_color: Vec3f::new(1.0, 1.0, 1.0),
            _pad1: [0; 4],
            specular_tint: Vec3f::new(1.0, 1.0, 1.0),
            _pad2: [0; 4],
            emission: Vec3f::new(0.0, 0.0, 0.0),
            transmission: 0.0,
            ior: 1.45,
            roughness: 1.0,
            metallic: 0.0,
            base_color_tex_id: u32::MAX,
            emission_tex_id: u32::MAX,
            _pad3: [0; 12],
        };
    }
}

#[derive(Clone, Default)]
pub struct Camera {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3f,
    forward: Vec3f,
    up: Vec3f,
    right: Vec3f,
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
