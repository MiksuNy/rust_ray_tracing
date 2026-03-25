use crate::{Vec3f, log_error, log_info, log_warning, scene::Material, texture::Texture};
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

        log_info!("Loading scene from '{}'", path);

        let start_time = std::time::Instant::now();

        let buffer = std::fs::read_to_string(path).unwrap();
        let lines = buffer.lines();

        let has_mtl: bool;
        let mtl_lib = lines
            .clone()
            .find(|line| line.trim_start().starts_with("mtllib"));
        if mtl_lib.is_some() {
            // FIXME: Add support for relative paths, for now this only works with absolute paths
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
            if let Some(prefix) = split.nth(0) {
                match prefix {
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
            }
        });

        // Triangles
        let mut active_material_id: u32 = 0;
        for line in lines {
            if line.trim_start().starts_with("usemtl ") && has_mtl {
                let mtl_name = line.strip_prefix("usemtl ").unwrap();
                let Some(mtl_id) = obj.material_names.iter().position(|name| name == mtl_name)
                else {
                    log_error!(
                        "While trying to set a material id for triangles, material with name '{}' doesn't exist",
                        mtl_name
                    );
                    continue;
                };
                active_material_id = mtl_id as u32;
            } else if line.trim_start().starts_with("f ") {
                let mut tri = Triangle::from_str(line.strip_prefix("f ").unwrap()).unwrap();
                tri.material_id = active_material_id;
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
        let mut lines = buffer.lines();

        while let Some(line) = lines.next() {
            if line.starts_with("newmtl ") {
                let mut material = Material::default();
                obj.material_names
                    .push(line.strip_prefix("newmtl ").unwrap().to_string());

                while let Some(line) = lines.next() {
                    let mut attribute = line.split_whitespace().into_iter();
                    // Consume the prefix so we can iterate only the data later
                    let Some(prefix) = attribute.nth(0) else {
                        break;
                    };

                    match prefix {
                        "Kd" => {
                            attribute.enumerate().for_each(|(i, val)| {
                                material.base_color.data[i] = val.parse().unwrap();
                            });
                        }
                        "Ks" => {
                            attribute.enumerate().for_each(|(i, val)| {
                                material.specular_tint.data[i] = val.parse().unwrap();
                            });
                        }
                        "Ke" => {
                            attribute.enumerate().for_each(|(i, val)| {
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
                        "d" => {
                            material.transparency = attribute.next().unwrap().parse().unwrap();
                        }
                        "map_Kd" => {
                            let texture_path = attribute.next().unwrap();
                            Self::load_texture(
                                texture_path,
                                obj,
                                &mut material,
                                TextureType::BaseColor,
                            );
                        }
                        "map_Ke" => {
                            let texture_path = attribute.next().unwrap();
                            Self::load_texture(
                                texture_path,
                                obj,
                                &mut material,
                                TextureType::Emission,
                            );
                        }
                        "map_d" => {
                            let texture_path = attribute.next().unwrap();
                            Self::load_texture(
                                texture_path,
                                obj,
                                &mut material,
                                TextureType::Transparency,
                            );
                        }
                        "map_Pr" => {
                            let texture_path = attribute.next().unwrap();
                            Self::load_texture(
                                texture_path,
                                obj,
                                &mut material,
                                TextureType::Roughness,
                            );
                        }
                        "map_Pm" => {
                            let texture_path = attribute.next().unwrap();
                            Self::load_texture(
                                texture_path,
                                obj,
                                &mut material,
                                TextureType::Metallic,
                            );
                        }
                        _ => continue,
                    }
                }

                obj.materials.push(material);
            }
        }
    }

    fn load_texture(path: &str, obj: &mut OBJ, material: &mut Material, texture_type: TextureType) {
        let Some(texture) = Texture::load(path) else {
            return;
        };

        let mut index: i32 = -1;
        for (i, other_texture) in obj.textures.iter().enumerate() {
            if texture.hash == other_texture.hash {
                index = i as i32;
                break;
            }
        }

        let tex_id: u32;
        if index == -1 {
            obj.textures.push(texture);
            log_info!("Loaded texture from '{}'", path);
            tex_id = (obj.textures.len() - 1) as u32;
        } else {
            tex_id = index as u32;
        }

        match texture_type {
            TextureType::BaseColor => {
                material.base_color_tex_id = tex_id;
            }
            TextureType::Emission => {
                material.emission_tex_id = tex_id;
            }
            TextureType::Transparency => {
                material.transparency_tex_id = tex_id;
            }
            TextureType::Roughness => {
                material.roughness_tex_id = tex_id;
            }
            TextureType::Metallic => {
                material.metallic_tex_id = tex_id;
            }
        }
    }
}

enum TextureType {
    BaseColor,
    Emission,
    Transparency,
    Roughness,
    Metallic,
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

impl FromStr for Triangle {
    type Err = ();

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
            let index = index_str.parse::<i32>().unwrap() - 1;
            if index < 0 {
                panic!("Tried to load negative indices from an OBJ file");
            }
            return index as usize;
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
