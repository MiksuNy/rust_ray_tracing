use crate::Vec3f;
use crate::bvh::BVH;
use crate::loader::obj::OBJ;

/// Representation of a 3D scene for use in the ray tracer.
#[derive(Clone, Default)]
pub struct Scene {
    pub tris: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub bvh: BVH,
}

impl Scene {
    pub fn load_from_obj(path: &str) -> Self {
        return OBJ::load(path).into();
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
                    normal: *obj
                        .vertex_buffer
                        .normals
                        .get(obj_tri.normals[i])
                        .unwrap_or(&[0.0; 3]),
                    tex_coord: *obj
                        .vertex_buffer
                        .tex_coords
                        .get(obj_tri.tex_coords[i])
                        .unwrap_or(&[0.0; 2]),
                };
            }
            scene
                .tris
                .push(Triangle::new(vertices, obj_tri.material_id));
        }

        scene.materials = obj.materials;

        BVH::build(&mut scene);

        return scene;
    }
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[derive(Clone, Copy, Default)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub material_id: usize,
}

impl Triangle {
    fn new(vertices: [Vertex; 3], material_id: usize) -> Self {
        return Self {
            vertices,
            material_id,
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

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub base_color: Vec3f,
    pub specular_tint: Vec3f,
    pub emission: Vec3f,
    pub transmission: f32,
    pub ior: f32,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            name: String::from("default_material"),
            base_color: Vec3f::new(1.0, 1.0, 1.0),
            specular_tint: Vec3f::new(1.0, 1.0, 1.0),
            emission: Vec3f::new(0.0, 0.0, 0.0),
            transmission: 0.0,
            ior: 1.45,
            roughness: 1.0,
            metallic: 0.0,
        };
    }
}
