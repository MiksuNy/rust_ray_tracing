use crate::{Vec3f, scene::Material};
use std::str::FromStr;

#[derive(Default)]
pub struct OBJ {
    pub tris: Vec<Triangle>,
    pub vertex_buffer: VertexBuffer,
    pub materials: Vec<Material>,
}

impl OBJ {
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
                let v_1 = Vec3f::from(vertex_buffer.positions[tri.positions[0]]);
                let v_2 = Vec3f::from(vertex_buffer.positions[tri.positions[1]]);
                let v_3 = Vec3f::from(vertex_buffer.positions[tri.positions[2]]);
                let u = v_2 - v_1;
                let v = v_3 - v_1;
                let n = Vec3f::cross(u, v).normalized();
                vertex_buffer.normals.push(n.data);
                tri.normals[0] = i;
                tri.normals[1] = i;
                tri.normals[2] = i;
            }
        }

        eprintln!(
            "'{}' took:\t{} ms to load",
            path,
            start_time.elapsed().as_millis()
        );

        return OBJ {
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

        // NOTE: Triangles may be specified with negative indices. I don't see a reason to support
        // this since AFAIK Blender doesn't do this either.
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
