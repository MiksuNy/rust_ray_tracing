use crate::vec3::Vec3;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        return Self {
            vertices: [p1, p2, p3],
        };
    }
}
