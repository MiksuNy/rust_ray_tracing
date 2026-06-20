use crate::{
    log_error, log_info, log_warning, math::vec::*, math::vec3::*, scene::Material,
    texture::Texture, texture::TextureType,
};
use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct OBJ {
    pub tris: Vec<Triangle>,
    pub vertex_buffer: VertexBuffer,
    pub materials: HashMap<String, Material>,
    pub textures: Vec<Texture>,
}

impl OBJ {
    pub fn load(path: &str) -> Self {
        let mut obj = OBJ::default();

        log_info!("Loading scene from '{}'", path);

        let start_time = std::time::Instant::now();

        let buffer = std::fs::read_to_string(path).unwrap();
        let lines = buffer.lines();

        let has_mtl: bool;
        if let Some(mtl_line) = lines
            .clone()
            .find(|line| line.trim_start().starts_with("mtllib"))
        {
            let mtl_path = mtl_line.strip_prefix("mtllib ").unwrap();
            if let Some(mtl_path) = Self::get_resource_path(path, mtl_path) {
                Self::load_mtl(&mut obj, mtl_path.as_str());
                has_mtl = true;
            } else {
                log_warning!(
                    "An mtllib line was found but the corresponding .mtl file was not found, using default material for scene"
                );
                obj.materials = HashMap::new();
                obj.materials
                    .insert("default_material".into(), Material::default());
                has_mtl = false;
            }
        } else {
            log_info!("No mtllib line found, using default material for scene");
            obj.materials = HashMap::new();
            obj.materials
                .insert("default_material".into(), Material::default());
            has_mtl = false;
        }

        let mut active_material_id: u32 = 0;
        for line in lines {
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
                    "usemtl" => {
                        if has_mtl {
                            let mtl_name = line.strip_prefix("usemtl ").unwrap();
                            let Some(mtl_id) =
                                obj.materials.iter().position(|(name, _)| name == mtl_name)
                            else {
                                log_error!(
                                    "While trying to set a material id for triangles, material with name '{}' doesn't exist",
                                    mtl_name
                                );
                                continue;
                            };
                            active_material_id = mtl_id as u32;
                        }
                    }
                    "f" => {
                        let triangles =
                            Triangle::from_str(line.strip_prefix("f ").unwrap()).unwrap();
                        for mut triangle in triangles {
                            triangle.material_id = active_material_id;
                            obj.tris.push(triangle);
                        }
                    }
                    _ => (),
                }
            }
        }

        // Precalculate vertex normals
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
                let mut new_material = (
                    line.strip_prefix("newmtl ").unwrap().to_string(),
                    Material::default(),
                );

                while let Some(line) = lines.next() {
                    let mut attribute = line.split_whitespace().into_iter();
                    // Consume the prefix so we can iterate only the data later
                    let Some(prefix) = attribute.nth(0) else {
                        break;
                    };

                    match prefix {
                        "Kd" => {
                            attribute.enumerate().for_each(|(i, val)| {
                                new_material.1.base_color.data[i] = val.parse().unwrap();
                            });
                        }
                        "Ks" => {
                            attribute.enumerate().for_each(|(i, val)| {
                                new_material.1.specular_tint.data[i] = val.parse().unwrap();
                            });
                        }
                        "Ke" => {
                            attribute.enumerate().for_each(|(i, val)| {
                                new_material.1.emission.data[i] = val.parse().unwrap();
                            });
                        }
                        "Ni" => {
                            new_material.1.ior = attribute.next().unwrap().parse().unwrap();
                        }
                        "Pr" => {
                            new_material.1.roughness = attribute.next().unwrap().parse().unwrap();
                        }
                        "Pm" => {
                            new_material.1.metallic = attribute.next().unwrap().parse().unwrap();
                        }
                        // NOTE: Blender exports "Tf" as a 3D vector, we only care about the
                        // first component. AFAIK the components are always the same.
                        "Tf" => {
                            new_material.1.transmission =
                                attribute.next().unwrap().parse().unwrap();
                        }
                        "d" => {
                            new_material.1.transparency =
                                attribute.next().unwrap().parse().unwrap();
                        }
                        "map_Kd" => {
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.next().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::BaseColor,
                                );
                            }
                        }
                        "map_d" => {
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.next().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::Transparency,
                                );
                            }
                        }
                        "map_Pr" => {
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.next().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::Roughness,
                                );
                            }
                        }
                        "map_Pm" => {
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.next().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::Metallic,
                                );
                            }
                        }
                        "map_Ke" => {
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.next().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::Emission,
                                );
                            }
                        }
                        "map_Bump" => {
                            // NOTE: Using .last() here because Blender also
                            // exports "bm" (bump map strength?) as a parameter but we don't use it
                            if let Some(texture_path) =
                                Self::get_resource_path(path, attribute.last().unwrap())
                            {
                                Self::load_texture(
                                    texture_path.as_str(),
                                    obj,
                                    &mut new_material.1,
                                    TextureType::Normal,
                                );
                            }
                        }
                        _ => continue,
                    }
                }

                obj.materials.insert(new_material.0, new_material.1);
            }
        }
    }

    fn load_texture(path: &str, obj: &mut OBJ, material: &mut Material, texture_type: TextureType) {
        let Some(texture) = Texture::load(path, texture_type) else {
            return;
        };

        let mut index: i32 = -1;
        for (i, other_texture) in obj.textures.iter().enumerate() {
            if texture.hash == other_texture.hash {
                index = i as i32;
                break;
            }
        }

        let tex_id: u8;
        if index == -1 {
            obj.textures.push(texture);
            log_info!("Loaded texture from '{}'", path);
            tex_id = (obj.textures.len() - 1) as u8;
        } else {
            tex_id = index as u8;
        }

        match texture_type {
            TextureType::BaseColor => {
                material.packed_tex_ids_1 &= u32::from_le_bytes([tex_id, 0xFF, 0xFF, 0xFF]);
            }
            TextureType::Transparency => {
                material.packed_tex_ids_1 &= u32::from_le_bytes([0xFF, tex_id, 0xFF, 0xFF]);
            }
            TextureType::Roughness => {
                material.packed_tex_ids_1 &= u32::from_le_bytes([0xFF, 0xFF, tex_id, 0xFF]);
            }
            TextureType::Metallic => {
                material.packed_tex_ids_1 &= u32::from_le_bytes([0xFF, 0xFF, 0xFF, tex_id]);
            }
            TextureType::Emission => {
                material.packed_tex_ids_2 &= u32::from_le_bytes([tex_id, 0xFF, 0xFF, 0xFF]);
            }
            TextureType::Normal => {
                material.packed_tex_ids_2 &= u32::from_le_bytes([0xFF, tex_id, 0xFF, 0xFF]);
            }
        }
    }

    /// Takes a path to the file the resource is referenced in, and a path to the actual resource
    /// itself.
    ///
    /// Returns an Option containing the path to the resource with the following rules:
    ///
    /// If resource_path is relative to file_path, returns file_path + resource_path.
    /// If resource_path is absolute, returns resource_path.
    /// Otherwise return None.
    fn get_resource_path(file_path: &str, resource_path: &str) -> Option<String> {
        let mut file_path_buf = PathBuf::from(file_path);
        let mut resource_path_buf = PathBuf::from(resource_path);

        if resource_path_buf.is_relative() {
            file_path_buf.pop();
            resource_path_buf = file_path_buf.join(resource_path_buf);
            return Some(resource_path_buf.to_string_lossy().to_string());
        } else if resource_path_buf.is_absolute() {
            return Some(resource_path_buf.to_string_lossy().to_string());
        }

        return None;
    }
}

/// Used to build final scene triangles from OBJ triangles
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

impl Triangle {
    pub fn from_str(s: &str) -> Result<Vec<Self>, ()> {
        // NOTE: Triangles may be specified with negative indices. I don't see a reason to support
        // this since AFAIK Blender doesn't do this either.
        let read_index = |index_str: &str| -> usize {
            let index = index_str.parse::<i32>().unwrap() - 1;
            if index < 0 {
                panic!("Tried to load negative indices from an OBJ file");
            }
            return index as usize;
        };

        let triangle_from_index_groups = |index_groups: &[&str; 3]| -> Self {
            let mut triangle = Triangle::default();

            for (group_id, group) in index_groups.iter().enumerate() {
                if group.contains("//") {
                    group
                        .split("//")
                        .enumerate()
                        .for_each(|(i, index_str)| match i {
                            0 => triangle.positions[group_id] = read_index(index_str),
                            1 => triangle.normals[group_id] = read_index(index_str),
                            _ => (),
                        });
                } else if group.contains("/") {
                    let split = group.split("/");
                    if split.clone().count() == 2 {
                        split.enumerate().for_each(|(i, index_str)| match i {
                            0 => triangle.positions[group_id] = read_index(index_str),
                            1 => triangle.tex_coords[group_id] = read_index(index_str),
                            _ => (),
                        });
                    } else if split.clone().count() == 3 {
                        split.enumerate().for_each(|(i, index_str)| match i {
                            0 => triangle.positions[group_id] = read_index(index_str),
                            1 => triangle.tex_coords[group_id] = read_index(index_str),
                            2 => triangle.normals[group_id] = read_index(index_str),
                            _ => (),
                        });
                    }
                } else {
                    group
                        .split(" ")
                        .for_each(|index_str| triangle.positions[group_id] = read_index(index_str));
                }
            }

            return triangle;
        };

        let index_groups = s.split_whitespace().collect::<Vec<&str>>();
        match index_groups.len() {
            // triangle
            3 => {
                return Ok(vec![triangle_from_index_groups(&[
                    index_groups[0],
                    index_groups[1],
                    index_groups[2],
                ])]);
            }
            // quad
            4 => {
                let index_groups_1 = [index_groups[0], index_groups[1], index_groups[3]];
                let index_groups_2 = [index_groups[1], index_groups[2], index_groups[3]];
                return Ok(vec![
                    triangle_from_index_groups(&index_groups_1),
                    triangle_from_index_groups(&index_groups_2),
                ]);
            }
            // n-gon
            5.. => {
                let mut triangles: Vec<Self> = vec![];
                for i in 0..index_groups.len() - 2 {
                    triangles.push(triangle_from_index_groups(&[
                        index_groups[0],
                        index_groups[i + 1],
                        index_groups[i + 2],
                    ]));
                }
                return Ok(triangles);
            }
            _ => Err(()),
        }
    }
}
