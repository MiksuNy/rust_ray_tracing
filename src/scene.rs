use crate::Vec3;
use crate::bvh::BVH;
use crate::scene::obj::Obj;

/// Representation of a 3D scene for use in the ray tracer.
#[derive(Clone, Default)]
pub struct Scene {
    pub tris: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub bvh: BVH,
}

impl Scene {
    pub fn load_from_obj(path: &str) -> Self {
        let obj = Obj::load(path);
        return Scene::from(obj);
    }
}

impl From<Obj> for Scene {
    fn from(obj: Obj) -> Self {
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

    pub fn mid(&self) -> Vec3 {
        return Vec3::new(
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
    pub base_color: Vec3,
    pub emission: Vec3,
    pub transmission: f32,
    pub ior: f32,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            name: String::from("default_material"),
            base_color: Vec3::new(1.0, 1.0, 1.0),
            emission: Vec3::new(0.0, 0.0, 0.0),
            transmission: 0.0,
            ior: 1.45,
            roughness: 1.0,
            metallic: 0.0,
        };
    }
}

/// Worst .obj parser ever
mod obj {
    use crate::{scene::Material, vec3::Vec3};
    use std::str::FromStr;

    #[derive(Default)]
    pub struct Obj {
        pub tris: Vec<Triangle>,
        pub vertex_buffer: VertexBuffer,
        pub materials: Vec<Material>,
    }

    impl Obj {
        pub fn load(path: &str) -> Self {
            let start_time = std::time::Instant::now();

            let buffer = std::fs::read_to_string(path).unwrap();
            let lines = buffer
                .lines()
                .filter(|line| !line.trim_start().starts_with("#"));

            let mut tris: Vec<Triangle> = Vec::new();
            let materials: Vec<Material>;

            let mtl_lib = lines
                .clone()
                .find(|line| line.trim_start().starts_with("mtllib"));
            if mtl_lib.is_some() {
                let mtl_name = mtl_lib.unwrap().strip_prefix("mtllib ").unwrap();
                let mut mtl_path = path.split_at(path.rfind("/").unwrap()).0.to_string();
                mtl_path.push('/');
                mtl_path.push_str(mtl_name);
                materials = Self::load_mtl(mtl_path.as_str());
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

            // Precalculating vertex normals
            if vertex_buffer.normals.is_empty() {
                for (i, tri) in tris.iter_mut().enumerate() {
                    let v_1 = Vec3::from(vertex_buffer.positions[tri.positions[0]]);
                    let v_2 = Vec3::from(vertex_buffer.positions[tri.positions[1]]);
                    let v_3 = Vec3::from(vertex_buffer.positions[tri.positions[2]]);
                    let u = Vec3::sub(v_2, v_1);
                    let v = Vec3::sub(v_3, v_1);
                    let n = Vec3::cross(u, v).normalized();
                    vertex_buffer.normals.push(n.data);
                    tri.normals[0] = i;
                    tri.normals[1] = i;
                    tri.normals[2] = i;
                }
            }

            println!(
                "Model loading took:\t{} ms",
                start_time.elapsed().as_millis()
            );

            return Obj {
                tris,
                vertex_buffer,
                materials,
            };
        }

        // TODO: Refactor this to be more like the .obj loading function.
        fn load_mtl(path: &str) -> Vec<Material> {
            let mut material_buffer: Vec<Material> = Vec::new();

            let buffer = std::fs::read_to_string(path).unwrap();
            let mut lines = buffer
                .lines()
                .filter(|line| !line.trim_start().starts_with("#"))
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
                            // NOTE: Blender exports "Tf" as a 3D vector, we only care about the
                            // first component. AFAIK the components are always the same.
                            "Tf" => {
                                material.transmission = attribute.next().unwrap().parse().unwrap();
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

    /// Used to build final scene triangles from .obj triangles
    #[derive(Default)]
    pub struct VertexBuffer {
        pub positions: Vec<[f32; 3]>,
        pub tex_coords: Vec<[f32; 2]>,
        pub normals: Vec<[f32; 3]>,
    }

    /// In a .obj file, triangles are represented as indices (f) to a buffer of vertex data (v, vn, vt)
    #[derive(Default)]
    pub struct Triangle {
        pub positions: [usize; 3],
        pub tex_coords: [usize; 3],
        pub normals: [usize; 3],
        pub material_id: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct ParseObjTriangleError;

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
