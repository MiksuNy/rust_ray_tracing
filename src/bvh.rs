use crate::{
    scene::{Scene, Triangle},
    vec3::Vec3,
};

#[derive(Clone, Default)]
pub struct BVH {
    pub nodes: Vec<Node>,
}

impl BVH {
    pub fn build(scene: &mut Scene) {
        let mut bvh = Self::default();
        let mut root = Node::default();
        for tri in &scene.tris {
            root.grow_by_tri(tri);
        }
        root.num_tris = scene.tris.len();
        bvh.nodes.push(root);

        Self::split_node(0, &mut bvh, scene);

        print!("\nBVH length:\t{}\n", bvh.nodes.len());

        scene.bvh = bvh;
    }

    fn split_node(index: usize, bvh: &mut Self, scene: &mut Scene) {
        let used_nodes = bvh.nodes.len();
        let node = bvh.nodes.get_mut(index).unwrap();

        // 4 triangles seems to be a good number for now
        if node.num_tris <= 4 {
            return;
        }

        let mut a = Node::default();
        let mut b = Node::default();

        let extent = node.extent();
        let mut split_axis: usize = 0;
        if extent.data[1] > extent.data[0] {
            split_axis = 1;
        } else if extent.data[2] > extent.data[split_axis] {
            split_axis = 2;
        }
        let split_pos = (node.bounds_min.data[split_axis] + node.bounds_max.data[split_axis]) * 0.5;

        // Sort triangles
        let mut i: usize = node.first_tri_id;
        let mut j: usize = i + node.num_tris - 1;
        while i <= j {
            if scene.tris[i].mid().data[split_axis] < split_pos {
                i += 1;
            } else {
                scene.tris.swap(i, j);
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
            a.grow_by_tri(&scene.tris[a.first_tri_id + i]);
        }
        for i in 0..b.num_tris {
            b.grow_by_tri(&scene.tris[b.first_tri_id + i]);
        }

        bvh.nodes.push(a);
        bvh.nodes.push(b);

        Self::split_node(used_nodes, bvh, scene);
        Self::split_node(used_nodes + 1, bvh, scene);
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
            bounds_min: Vec3::from(f32::MAX),
            bounds_max: Vec3::from(-f32::MAX),
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
                self.bounds_min.data[i] = f32::min(self.bounds_min.data[i], vert.position[i]);
                self.bounds_max.data[i] = f32::max(self.bounds_max.data[i], vert.position[i]);
            }
        });
    }

    fn extent(&self) -> Vec3 {
        return Vec3::sub(self.bounds_max, self.bounds_min);
    }

    fn surface_area(&self) -> f32 {
        return 2.0 * (self.extent().data[0] * self.extent().data[2])
            + 2.0 * (self.extent().data[0] * self.extent().data[1])
            + 2.0 * (self.extent().data[2] * self.extent().data[1]);
    }
}
