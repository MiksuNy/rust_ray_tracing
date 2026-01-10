use crate::{Vec3f, log_info, log_warning, scene::Material, texture::Texture};
use std::str::FromStr;

#[derive(Default)]
pub struct OBJ {
    pub tris: Vec<Triangle>,
    pub vertex_buffer: VertexBuffer,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    material_names: Vec<String>,
}

impl OBJ {
    pub fn load(path: &str) -> Self {
        let mut obj = OBJ::default();

        let start_time = std::time::Instant::now();

        let buffer = std::fs::read_to_string(path).unwrap();
        let lines = buffer
            .lines()
            .filter(|line| !line.trim_start().starts_with("#"));

        let has_mtl: bool;
        let mtl_lib = lines
            .clone()
            .find(|line| line.trim_start().starts_with("mtllib"));
        if mtl_lib.is_some() {
            // FIXME: This is really stupid, .mtl paths can be relative to the .obj directory so we
            // prepend the .mtl path with the .obj path
            let mtl_name = mtl_lib.unwrap().strip_prefix("mtllib ").unwrap();
            let mut mtl_path = path.split_at(path.rfind("/").unwrap()).0.to_string();
            mtl_path.push('/');
            mtl_path.push_str(mtl_name);
            let mtl_path_str = mtl_path.as_str();

            if !std::fs::exists(mtl_path_str).unwrap() {
                log_warning!(
                    "An mtllib line was found but the corresponding .mtl file was not found, using default material for scene"
                );
                obj.materials = vec![Material::default()];
                has_mtl = false;
            } else {
                Self::load_mtl(&mut obj, mtl_path_str);
                has_mtl = true;
            }
        } else {
            log_info!("No mtllib found, using default material for scene");
            obj.materials = vec![Material::default()];
            has_mtl = false;
        }

        lines.clone().for_each(|line| {
            let mut split = line.split_whitespace();
            match split.nth(0).unwrap() {
                "v" => {
                    let mut data: [f32; 3] = [0.0; 3];
                    for (i, value) in split.enumerate() {
                        data[i] = value.parse::<f32>().unwrap();
                    }
                    obj.vertex_buffer.positions.push(data);
                }
                "vt" => {
                    let mut data: [f32; 2] = [0.0; 2];
                    for (i, value) in split.enumerate() {
                        data[i] = value.parse::<f32>().unwrap();
                    }
                    obj.vertex_buffer.tex_coords.push(data);
                }
                "vn" => {
                    let mut data: [f32; 3] = [0.0; 3];
                    for (i, value) in split.enumerate() {
                        data[i] = value.parse::<f32>().unwrap();
                    }
                    obj.vertex_buffer.normals.push(data);
                }
                _ => (),
            }
        });

        // Triangles
        let mut active_material_id: usize = 0;
        for line in lines {
            if line.trim_start().starts_with("usemtl ") && has_mtl {
                active_material_id = obj
                    .material_names
                    .iter()
                    .position(|mtl_name| mtl_name == line.strip_prefix("usemtl ").unwrap())
                    .unwrap();
            } else if line.trim_start().starts_with("f ") {
                let mut tri = Triangle::from_str(line.strip_prefix("f ").unwrap()).unwrap();
                tri.material_id = active_material_id as u32;
                obj.tris.push(tri);
            }
        }

        // Precalculating vertex normals
        if obj.vertex_buffer.normals.is_empty() {
            for (i, tri) in obj.tris.iter_mut().enumerate() {
                let v_1 = Vec3f::from(obj.vertex_buffer.positions[tri.positions[0]]);
                let v_2 = Vec3f::from(obj.vertex_buffer.positions[tri.positions[1]]);
                let v_3 = Vec3f::from(obj.vertex_buffer.positions[tri.positions[2]]);
                let u = v_2 - v_1;
                let v = v_3 - v_1;
                let n = Vec3f::cross(u, v).normalized();
                obj.vertex_buffer.normals.push(n.data);
                tri.normals[0] = i;
                tri.normals[1] = i;
                tri.normals[2] = i;
            }
        }

        log_info!(
            "'{}' took {} ms to load\n",
            path,
            start_time.elapsed().as_millis()
        );

        return obj;
    }

    fn load_mtl(obj: &mut OBJ, path: &str) {
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
                obj.material_names
                    .push(line.strip_prefix("newmtl ").unwrap().to_string());

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
                        "Ks" => {
                            attribute.into_iter().enumerate().for_each(|(i, val)| {
                                material.specular_tint.data[i] = val.parse().unwrap();
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
                        // NOTE: Materials can have duplicate textures, this might not be a problem
                        // for smaller scenes but maybe we should check if a texture is already in
                        // memory and just use that instead of duplicating?
                        "map_Kd" => {
                            let texture = Texture::load(attribute.next().unwrap());
                            if texture.is_some() {
                                obj.textures.push(texture.unwrap());
                                material.base_color_tex_id = (obj.textures.len() - 1) as u32;
                            }
                        }
                        "map_Ke" => {
                            let texture = Texture::load(attribute.next().unwrap());
                            if texture.is_some() {
                                obj.textures.push(texture.unwrap());
                                material.emission_tex_id = (obj.textures.len() - 1) as u32;
                            }
                        }
                        _ => continue,
                    }
                }

                obj.materials.push(material);
            }
        }
    }
}

/// Used to build final scene triangles from .obj triangles
#[derive(Default)]
pub struct VertexBuffer {
    pub positions: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
}

/// In a .obj file, triangles are represented as indices (f) to a buffer of vertex data (v, vt, vn)
#[derive(Default)]
pub struct Triangle {
    pub positions: [usize; 3],
    pub tex_coords: [usize; 3],
    pub normals: [usize; 3],
    pub material_id: u32,
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
    /// v/vt v/vt v/vt
    /// v v v
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut triangle = Triangle::default();

        // NOTE: Triangles may be specified with negative indices. I don't see a reason to support
        // this since AFAIK Blender doesn't do this either.
        let read_index = |index_str: &str| -> usize {
            if index_str.parse::<i32>().unwrap() < 1 {
                panic!("Negative indices are not supported for OBJ!");
            }
            return index_str.parse::<usize>().unwrap() - 1;
        };

        let vertices = s.split_whitespace();
        for (vertex_id, vertex) in vertices.enumerate() {
            if vertex.contains("//") {
                vertex
                    .split("//")
                    .enumerate()
                    .for_each(|(i, index_str)| match i {
                        0 => triangle.positions[vertex_id] = read_index(index_str),
                        1 => triangle.normals[vertex_id] = read_index(index_str),
                        _ => (),
                    });
            } else if vertex.contains("/") {
                let split = vertex.split("/");
                if split.clone().count() == 2 {
                    split.enumerate().for_each(|(i, index_str)| match i {
                        0 => triangle.positions[vertex_id] = read_index(index_str),
                        1 => triangle.tex_coords[vertex_id] = read_index(index_str),
                        _ => (),
                    });
                } else if split.clone().count() == 3 {
                    split.enumerate().for_each(|(i, index_str)| match i {
                        0 => triangle.positions[vertex_id] = read_index(index_str),
                        1 => triangle.tex_coords[vertex_id] = read_index(index_str),
                        2 => triangle.normals[vertex_id] = read_index(index_str),
                        _ => (),
                    });
                }
            } else {
                vertex
                    .split(" ")
                    .for_each(|index_str| triangle.positions[vertex_id] = read_index(index_str));
            }
        }

        return Ok(triangle);
    }
}
