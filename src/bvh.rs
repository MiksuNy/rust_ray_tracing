use crate::{
    obj::{Model, Triangle},
    vec3::Vec3,
};

#[derive(Clone)]
pub struct BVH {
    pub nodes: Vec<Node>,
}

impl BVH {
    pub fn new() -> Self {
        return Self { nodes: Vec::new() };
    }

    pub fn build(model: &mut Model, depth: usize) -> Self {
        if depth == 0 {
            panic!("Can't create a BVH with depth of 0");
        }

        let mut bvh = Self::new();
        let mut root = Node::default();
        for tri in &model.tris {
            root.grow_by_tri(tri);
            root.num_tris += 1;
        }
        root.children_id = 1;
        bvh.nodes.push(root);

        // TODO: Here goes the *actual* BVH generation

        return bvh;
    }
}

#[derive(Clone, Copy)]
pub struct Node {
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub children_id: usize,
    pub first_tri_id: usize,
    pub num_tris: usize,
}

impl Default for Node {
    fn default() -> Self {
        return Self {
            bounds_min: Vec3::from_f32(f32::MAX),
            bounds_max: Vec3::from_f32(-f32::MAX),
            children_id: 0,
            first_tri_id: 0,
            num_tris: 0,
        };
    }
}

impl Node {
    fn grow_by_tri(&mut self, tri: &Triangle) {
        tri.vertices.iter().for_each(|vert| {
            for i in 0..3 {
                self.bounds_min.data[i] = f32::min(self.bounds_min.data[i], vert.data[i]);
                self.bounds_max.data[i] = f32::max(self.bounds_max.data[i], vert.data[i]);
            }
        });
    }

    /// Used for determining the axis on which to split a node when generating the BVH
    fn longest_axis(&self) -> usize {
        let axis: usize;
        let extent = Vec3::add(Vec3::abs(self.bounds_min), Vec3::abs(self.bounds_max));
        if extent.data[1] > extent.data[0] {
            axis = 1;
        } else if extent.data[2] > extent.data[1] {
            axis = 2;
        } else {
            axis = 0;
        }
        return axis;
    }
}
