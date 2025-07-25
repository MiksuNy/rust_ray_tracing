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

    pub fn build(model: &mut Model) {
        let mut bvh = Self::new();
        let mut root = Node::default();
        for tri in &model.tris {
            root.grow_by_tri(tri);
            root.num_tris += 1;
        }
        bvh.nodes.push(root);

        Self::split_node(0, &mut bvh, model);

        model.bvh = bvh;
    }

    fn split_node(index: usize, bvh: &mut Self, model: &mut Model) {
        let used_nodes = bvh.nodes.len();
        let node = bvh.nodes.get_mut(index).unwrap();
        if node.num_tris <= 2 {
            return;
        }

        let mut a = Node::default();
        let mut b = Node::default();

        let extent = Vec3::sub(node.bounds_max, node.bounds_min);
        let mut split_axis: usize = 0;
        if extent.data[1] > extent.data[0] {
            split_axis = 1;
        } else if extent.data[2] > extent.data[split_axis] {
            split_axis = 2;
        }
        let split_pos = (node.bounds_min.data[split_axis] + extent.data[split_axis]) / 2.0;

        let mut i: usize = node.first_tri_id;
        let mut j: usize = i + node.num_tris - 1;
        while i <= j {
            if model.tris[i].mid().data[split_axis] < split_pos {
                i += 1;
            } else {
                model.tris.swap(i, j);
                j -= 1;
            }
        }

        let a_count = i - node.first_tri_id;
        if a_count == 0 || a_count == node.num_tris {
            return;
        }

        a.first_tri_id = node.first_tri_id;
        a.num_tris = a_count;
        b.first_tri_id = i;
        b.num_tris = node.num_tris - a_count;
        node.children_id = used_nodes;
        node.num_tris = 0;

        for i in 0..a.num_tris {
            a.grow_by_tri(&model.tris[a.first_tri_id + i]);
        }
        for i in 0..b.num_tris {
            b.grow_by_tri(&model.tris[b.first_tri_id + i]);
        }

        bvh.nodes.push(a);
        bvh.nodes.push(b);

        Self::split_node(used_nodes, bvh, model);
        Self::split_node(used_nodes + 1, bvh, model);
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
}
