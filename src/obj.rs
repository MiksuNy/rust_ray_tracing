use crate::Vec3;
use crate::bvh::BVH;
use std::fs;

#[derive(Clone)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub material_id: usize,
}

impl Triangle {
    fn new(p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], material_id: usize) -> Self {
        return Self {
            vertices: [
                Vec3::from_array(p1),
                Vec3::from_array(p2),
                Vec3::from_array(p3),
            ],
            material_id,
        };
    }

    pub fn mid(&self) -> Vec3 {
        return Vec3::from_array(
            Vec3::new(
                (self.vertices[0].data[0] + self.vertices[1].data[0] + self.vertices[2].data[0])
                    / 3.0,
                (self.vertices[0].data[1] + self.vertices[1].data[1] + self.vertices[2].data[1])
                    / 3.0,
                (self.vertices[0].data[2] + self.vertices[1].data[2] + self.vertices[2].data[2])
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
            roughness: 0.5,
            metallic: 0.0,
        };
    }
}

#[derive(Clone)]
pub struct Model {
    pub tris: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub bvh: BVH,
}

impl Model {
    fn new() -> Self {
        return Self {
            tris: Vec::new(),
            materials: Vec::new(),
            bvh: BVH::new(),
        };
    }

    /// Parses a .obj file and optionally a .mtl file, returns a Model.
    /// If mtl_path is None, creates a default material which all triangles then use.
    /// i.e., model.materials will always have a length of at least 1.
    pub fn load(obj_path: &str, mtl_path: Option<&str>) -> Model {
        let mut model = Model::new();

        // Read the .mtl file or create a default material for all tris
        if mtl_path.is_some() {
            model.materials = Self::load_mtl(mtl_path.unwrap());
        } else {
            model.materials.push(Material::default());
        }

        let binding = fs::read_to_string(obj_path).unwrap();
        let lines = binding
            .lines()
            .filter(|line| !line.trim_start().starts_with("#"));

        // Make temporary buffers for all vertex information so we can construct the vertices later
        let mut position_buffer: Vec<[f32; 3]> = Vec::new();
        let mut tex_coord_buffer: Vec<[f32; 2]> = Vec::new();
        let mut normal_buffer: Vec<[f32; 3]> = Vec::new();

        // Vertices
        let mut v_lines = lines.clone();
        loop {
            let line = v_lines.next();
            if line.is_none() {
                break;
            }

            match line.unwrap().split_whitespace().nth(0).unwrap() {
                "v" => {
                    let mut data = [0.0f32; 3];
                    line.unwrap()
                        .strip_prefix("v ")
                        .unwrap()
                        .split_whitespace()
                        .enumerate()
                        .for_each(|(i, val)| {
                            data[i] = val.parse().unwrap();
                        });
                    position_buffer.push(data);
                }
                "vt" => {
                    let mut data = [0.0f32; 2];
                    line.unwrap()
                        .strip_prefix("vt ")
                        .unwrap()
                        .split_whitespace()
                        .enumerate()
                        .for_each(|(i, val)| {
                            data[i] = val.parse().unwrap();
                        });
                    tex_coord_buffer.push(data);
                }
                "vn" => {
                    let mut data = [0.0f32; 3];
                    line.unwrap()
                        .strip_prefix("vn ")
                        .unwrap()
                        .split_whitespace()
                        .enumerate()
                        .for_each(|(i, val)| {
                            data[i] = val.parse().unwrap();
                        });
                    normal_buffer.push(data);
                }
                _ => (),
            }
        }

        // Indices
        let mut i_lines = lines.clone();
        let mut active_material_id: usize = 0;
        loop {
            let Some(line) = i_lines.next() else {
                break;
            };

            match line.split_whitespace().nth(0).unwrap() {
                "usemtl" => {
                    let name = line.strip_prefix("usemtl ").unwrap();
                    active_material_id = model
                        .materials
                        .iter()
                        .position(|mtl| mtl.name.as_str() == name)
                        .unwrap_or(0);
                }
                "f" => {
                    let stripped = line.strip_prefix("f ").unwrap();

                    let mut position_indices: [usize; 3] = [0; 3];
                    let mut tex_coord_indices: [usize; 3] = [0; 3];
                    let mut normal_indices: [usize; 3] = [0; 3];

                    if stripped.contains("//") {
                        let split = stripped.split_whitespace().enumerate();
                        for (id, group) in split {
                            group.split("//").enumerate().for_each(|(i, val)| {
                                match i {
                                    0 => position_indices[id] = val.parse::<usize>().unwrap() - 1,
                                    1 => normal_indices[id] = val.parse::<usize>().unwrap() - 1,
                                    _ => (),
                                };
                            });
                        }
                    } else if stripped.contains("/") {
                        let split = stripped.split_whitespace().enumerate();
                        for (id, group) in split {
                            group.split("/").enumerate().for_each(|(i, val)| {
                                match i {
                                    0 => position_indices[id] = val.parse::<usize>().unwrap() - 1,
                                    1 => tex_coord_indices[id] = val.parse::<usize>().unwrap() - 1,
                                    2 => normal_indices[id] = val.parse::<usize>().unwrap() - 1,
                                    _ => (),
                                };
                            });
                        }
                    } else {
                        stripped
                            .split_whitespace()
                            .enumerate()
                            .for_each(|(i, val)| {
                                position_indices[i] = val.parse::<usize>().unwrap() - 1
                            });
                    }

                    let tri = Triangle::new(
                        position_buffer[position_indices[0]],
                        position_buffer[position_indices[1]],
                        position_buffer[position_indices[2]],
                        active_material_id,
                    );
                    model.tris.push(tri);
                }
                _ => (),
            }
        }

        BVH::build(&mut model);

        return model;
    }

    fn load_mtl(file_path: &str) -> Vec<Material> {
        let mut material_buffer: Vec<Material> = Vec::new();

        let binding = fs::read_to_string(file_path).unwrap();
        let mut lines = binding.lines().peekable();

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
