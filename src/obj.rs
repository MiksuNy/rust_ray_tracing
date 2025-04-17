use std::fs;

/// .obj files can contain positions (v), texture coordinates (vt), and normals (vn) in that order.
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

/// A triangle contains the indices (f) of each vertex position, texture coordinate and normal.
#[derive(Clone, Copy)]
pub struct Triangle {
    pub position_indices: [usize; 3],
    pub tex_coord_indices: [usize; 3],
    pub normal_indices: [usize; 3],
}

impl Triangle {
    fn new() -> Self {
        return Self {
            position_indices: [0; 3],
            tex_coord_indices: [0; 3],
            normal_indices: [0; 3],
        };
    }
}

#[derive(Clone)]
pub struct Model {
    pub vertex_buffer: Vec<Vertex>,
    pub triangle_buffer: Vec<Triangle>,
}

impl Model {
    fn new() -> Self {
        return Self {
            vertex_buffer: Vec::new(),
            triangle_buffer: Vec::new(),
        };
    }
}

pub fn load(file_path: &str) -> Model {
    let binding = fs::read_to_string(file_path).unwrap();
    let lines = binding.lines();

    // Make temporary buffers for all vertex information so we can construct the vertices later.
    let mut position_buffer: Vec<[f32; 3]> = Vec::new();
    let mut tex_coord_buffer: Vec<[f32; 2]> = Vec::new();
    let mut normal_buffer: Vec<[f32; 3]> = Vec::new();

    let mut positions: Vec<&str> = Vec::new();
    let mut tex_coords: Vec<&str> = Vec::new();
    let mut normals: Vec<&str> = Vec::new();
    lines.clone().for_each(|str| {
        if str.contains("v ") {
            positions.push(str.strip_prefix("v ").unwrap());
        } else if str.contains("vt ") {
            tex_coords.push(str.strip_prefix("vt ").unwrap());
        } else if str.contains("vn ") {
            normals.push(str.strip_prefix("vn ").unwrap());
        }
    });
    for str in positions {
        let mut data: [f32; 3] = [0.0; 3];
        str.split_whitespace().enumerate().for_each(|(i, val)| {
            data[i] = val.parse::<f32>().expect("Error parsing position data!!!")
        });
        position_buffer.push(data);
    }
    for str in tex_coords {
        let mut data: [f32; 2] = [0.0; 2];
        str.split_whitespace().enumerate().for_each(|(i, val)| {
            data[i] = val
                .parse::<f32>()
                .expect("Error parsing texture coordinate data!!!")
        });
        tex_coord_buffer.push(data);
    }
    for str in normals {
        let mut data: [f32; 3] = [0.0; 3];
        str.split_whitespace().enumerate().for_each(|(i, val)| {
            data[i] = val.parse::<f32>().expect("Error parsing normal data!!!")
        });
        normal_buffer.push(data);
    }

    let mut triangle_buffer: Vec<Triangle> = Vec::new();
    let mut triangles: Vec<&str> = Vec::new();
    lines.clone().for_each(|str| {
        if str.contains("f ") {
            triangles.push(str.strip_prefix("f ").unwrap());
        }
    });
    for str in &triangles {
        let mut triangle = Triangle::new();
        let vertices = str.split_whitespace();
        for vertex in vertices.enumerate() {
            if vertex.1.contains("//") {
                vertex
                    .1
                    .split("//")
                    .enumerate()
                    .for_each(|(i, val)| match i {
                        0 => {
                            triangle.position_indices[vertex.0] = val.parse::<usize>().unwrap() - 1
                        }
                        1 => triangle.normal_indices[vertex.0] = val.parse::<usize>().unwrap() - 1,
                        _ => (),
                    });
            } else if vertex.1.contains("/") {
                vertex
                    .1
                    .split("/")
                    .enumerate()
                    .for_each(|(i, val)| match i {
                        0 => {
                            triangle.position_indices[vertex.0] = val.parse::<usize>().unwrap() - 1
                        }
                        1 => {
                            triangle.tex_coord_indices[vertex.0] = val.parse::<usize>().unwrap() - 1
                        }
                        2 => triangle.normal_indices[vertex.0] = val.parse::<usize>().unwrap() - 1,
                        _ => (),
                    });
            } else {
                vertex
                    .1
                    .split(" ")
                    .enumerate()
                    .for_each(|(i, val)| match i {
                        0 => {
                            triangle.position_indices[vertex.0] = val.parse::<usize>().unwrap() - 1
                        }
                        _ => (),
                    });
            }
        }
        triangle_buffer.push(triangle);
    }

    let mut model = Model::new();
    model.triangle_buffer = triangle_buffer;
    for tri in model.triangle_buffer.iter() {
        for vertex in 0..3 {
            model.vertex_buffer.push(Vertex {
                position: position_buffer[tri.position_indices[vertex]],
                tex_coord: tex_coord_buffer[tri.tex_coord_indices[vertex]],
                normal: normal_buffer[tri.normal_indices[vertex]],
            });
        }
    }

    return model;
}
