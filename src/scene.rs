use crate::Vec3;
use crate::bvh::BVH;
use crate::scene::obj::Obj;
use std::fs;

/// Representation of a 3D scene for use in the ray tracer.
#[derive(Clone)]
pub struct Scene {
    pub tris: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub bvh: BVH,
}

impl Scene {
    fn new() -> Self {
        return Self {
            tris: Vec::new(),
            materials: Vec::new(),
            bvh: BVH::new(),
        };
    }
}

impl From<Obj> for Scene {
    fn from(value: Obj) -> Self {}
}

#[derive(Clone, Copy, Default)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
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

    pub fn mid(&self) -> Vec3 {
        return Vec3::from_array(
            Vec3::new(
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
            )
            .data,
        );
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub base_color: Vec3,
    pub emission: Vec3,
    pub transmission: Vec3,
    pub ior: f32,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            name: String::from("default_material"),
            base_color: Vec3::new(0.8, 0.8, 0.8),
            emission: Vec3::new(0.0, 0.0, 0.0),
            transmission: Vec3::new(0.0, 0.0, 0.0),
            ior: 1.45,
            roughness: 0.8,
            metallic: 0.0,
        };
    }
}

/// Worst .obj parser ever
mod obj {
    use crate::scene::Material;
    use std::str::FromStr;

    #[derive(Default)]
    pub struct Obj {
        tris: Vec<Triangle>,
        vertex_buffer: VertexBuffer,
        materials: Vec<Material>,
    }

    impl Obj {
        pub fn load(path: &str) -> Self {
            let Ok(buffer) = std::fs::read_to_string(path) else {
                panic!("Could not read .obj file at: '{path}'");
            };
            let lines = buffer
                .lines()
                .filter(|line| line.trim_start().starts_with("#"));

            let mut tris: Vec<Triangle> = Vec::new();
            let materials: Vec<Material>;

            let mtl_lib = lines
                .clone()
                .find(|line| line.trim_start().starts_with("mtllib"));
            if mtl_lib.is_some() {
                let mtl_path = mtl_lib.unwrap().strip_prefix("mtllib ").unwrap();
                materials = Self::load_mtl(mtl_path);
            } else {
                materials = vec![Material::default()];
            }

            // Vertices
            let mut vertex_buffer = VertexBuffer::default();
            lines.clone().for_each(|line| {
                let mut split = line.split_whitespace();
                match split.nth(0).unwrap() {
                    "v" => {
                        let mut data: [f32; 3] = [0.0; 3];
                        for (i, value) in split.enumerate() {
                            data[i] = value.parse::<f32>().unwrap();
                        }
                        vertex_buffer.positions.push(data);
                    }
                    "vt" => {
                        let mut data: [f32; 2] = [0.0; 2];
                        for (i, value) in split.enumerate() {
                            data[i] = value.parse::<f32>().unwrap();
                        }
                        vertex_buffer.tex_coords.push(data);
                    }
                    "vn" => {
                        let mut data: [f32; 3] = [0.0; 3];
                        for (i, value) in split.enumerate() {
                            data[i] = value.parse::<f32>().unwrap();
                        }
                        vertex_buffer.normals.push(data);
                    }
                    _ => (),
                }
            });

            // Triangles
            let mut active_material_id: usize = 0;
            for line in lines {
                if line.trim_start().starts_with("usemtl ") {
                    active_material_id = materials
                        .iter()
                        .position(|mtl| mtl.name == line.strip_prefix("usemtl ").unwrap())
                        .unwrap();
                } else if line.trim_start().starts_with("f ") {
                    let mut tri = Triangle::from_str(line.strip_prefix("f ").unwrap()).unwrap();
                    tri.material_id = active_material_id;
                    tris.push(tri);
                }
            }

            // TODO: If vertex normals are not present in the .obj file, we should probably
            // precalculate them from the positions anyway at this point.

            return Obj {
                tris,
                vertex_buffer,
                materials,
            };
        }

        // TODO: Refactor this to be more like the .obj loading function.
        fn load_mtl(path: &str) -> Vec<Material> {
            let mut material_buffer: Vec<Material> = Vec::new();

            let Ok(buffer) = std::fs::read_to_string(path) else {
                panic!("Could not read .mtl file at: '{path}'");
            };
            let mut lines = buffer
                .lines()
                .filter(|line| line.trim_start().starts_with("#"))
                .peekable();

            loop {
                let Some(line) = lines.next() else {
                    break;
                };

                if line.contains("newmtl") {
                    let mut material = Material::default();
                    material.name = line.strip_prefix("newmtl ").unwrap().to_string();

                    loop {
                        if lines.peek().is_none() {
                            break;
                        }

                        let mut attribute = lines.next().unwrap().split_whitespace();
                        // Consume the prefix so we can iterate only the data later
                        let Some(prefix) = attribute.nth(0) else {
                            break;
                        };

                        match prefix {
                            "Kd" => {
                                attribute.into_iter().enumerate().for_each(|(i, val)| {
                                    material.base_color.data[i] = val.parse().unwrap();
                                });
                            }
                            "Ke" => {
                                attribute.into_iter().enumerate().for_each(|(i, val)| {
                                    material.emission.data[i] = val.parse().unwrap();
                                });
                            }
                            "Ni" => {
                                material.ior = attribute.next().unwrap().parse().unwrap();
                            }
                            "Pr" => {
                                material.roughness = attribute.next().unwrap().parse().unwrap();
                            }
                            "Pm" => {
                                material.metallic = attribute.next().unwrap().parse().unwrap();
                            }
                            "Tf" => {
                                attribute.into_iter().enumerate().for_each(|(i, val)| {
                                    material.transmission.data[i] = val.parse().unwrap();
                                });
                            }
                            _ => continue,
                        }
                    }

                    material_buffer.push(material);
                }
            }

            return material_buffer;
        }
    }

    #[derive(Default)]
    struct VertexBuffer {
        positions: Vec<[f32; 3]>,
        tex_coords: Vec<[f32; 2]>,
        normals: Vec<[f32; 3]>,
    }

    /// In a .obj file, triangles are represented as indices (f) to a buffer of vertex data (v, vn, vt)
    #[derive(Default)]
    struct Triangle {
        positions: [usize; 3],
        tex_coords: [usize; 3],
        normals: [usize; 3],
        material_id: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ParseObjTriangleError;

    impl FromStr for Triangle {
        type Err = ParseObjTriangleError;

        /// Parses a group of vertices and returns a .obj representation of a triangle.
        ///
        /// Acceptable formats are:
        /// v/vt/vn v/vt/vn v/vt/vn
        /// v//vn v//vn v//vn
        /// v v v
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut triangle = Triangle::default();

            let vertices = s.split_whitespace();

            // TODO: Triangles may be specified with negative indices (WHY???) in the .obj format, this isn't handled
            // properly yet when parsing.
            for (vertex_id, vertex) in vertices.enumerate() {
                if vertex.contains("//") {
                    vertex.split("//").enumerate().for_each(|(i, str)| match i {
                        0 => triangle.positions[vertex_id] = str.parse::<usize>().unwrap() - 1,
                        1 => triangle.normals[vertex_id] = str.parse::<usize>().unwrap() - 1,
                        _ => (),
                    });
                } else if vertex.contains("/") {
                    vertex.split("/").enumerate().for_each(|(i, str)| match i {
                        0 => triangle.positions[vertex_id] = str.parse::<usize>().unwrap() - 1,
                        1 => triangle.tex_coords[vertex_id] = str.parse::<usize>().unwrap() - 1,
                        2 => triangle.normals[vertex_id] = str.parse::<usize>().unwrap() - 1,
                        _ => (),
                    });
                } else {
                    vertex.split(" ").for_each(|str| {
                        triangle.positions[vertex_id] = str.parse::<usize>().unwrap() - 1
                    });
                }
            }

            return Ok(triangle);
        }
    }
}
