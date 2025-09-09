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
        let start_time = std::time::Instant::now();

        let mut bvh = Self::default();
        let mut root = Node::default();
        for tri in &scene.tris {
            root.grow_by_tri(tri);
        }
        root.num_tris = scene.tris.len();
        bvh.nodes.push(root);

        Self::split_node(0, &mut bvh, scene);

        println!("BVH length:\t{}", bvh.nodes.len());
        println!(
            "BVH took:\t{} ms to build",
            start_time.elapsed().as_millis()
        );

        scene.bvh = bvh;
    }

    fn split_node(index: usize, bvh: &mut Self, scene: &mut Scene) {
        let used_nodes = bvh.nodes.len();
        let node = bvh.nodes.get_mut(index).unwrap();

        if node.num_tris <= 4 {
            return;
        }

        let parent_cost = node.num_tris as f32 * node.surface_area();

        // Surface area heuristic
        const NUM_BINS: usize = 16;
        let mut best_split_axis: usize = 0;
        let mut best_split_pos: f32 = 0.0;
        let mut best_split_cost: f32 = f32::MAX;
        for split_axis in 0..3 {
            let scale = node.extent().data[split_axis] / NUM_BINS as f32;
            for i in 0..NUM_BINS {
                let split_pos = node.bounds_min.data[split_axis] + i as f32 * scale;
                let split_cost = Self::evaluate_sah(scene, node, split_axis, split_pos);
                if split_cost < best_split_cost {
                    best_split_axis = split_axis;
                    best_split_pos = split_pos;
                    best_split_cost = split_cost;
                }
            }
        }

        if best_split_cost >= parent_cost {
            return;
        }

        let split_axis = best_split_axis;
        let split_pos = best_split_pos;

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

        let mut a = Node::default();
        let mut b = Node::default();
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

    fn evaluate_sah(scene: &Scene, node: &Node, split_axis: usize, split_pos: f32) -> f32 {
        let mut left = Node::default();
        let mut right = Node::default();

        for i in 0..node.num_tris {
            let tri = &scene.tris[node.first_tri_id + i];
            if tri.mid().data[split_axis] < split_pos {
                left.grow_by_tri(tri);
                left.num_tris += 1;
            } else {
                right.grow_by_tri(tri);
                right.num_tris += 1;
            }
        }

        let cost = left.num_tris as f32 * left.surface_area()
            + right.num_tris as f32 * right.surface_area();

        if cost > 0.0 {
            return cost;
        } else {
            return f32::MAX;
        }
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
        let extent = self.extent();
        return (extent.data[0] * extent.data[2])
            + (extent.data[0] * extent.data[1])
            + (extent.data[2] * extent.data[1]);
    }
}
