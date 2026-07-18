use crate::{
    log_info,
    math::{
        vec::{Max, Min},
        vec3::*,
    },
    scene::{Scene, Triangle},
};

#[derive(Clone, Default)]
pub struct BVH {
    pub nodes: Vec<Node>,
}

impl BVH {
    pub fn build(scene: &mut Scene) {
        log_info!("Building BVH for scene");

        let start_time = std::time::Instant::now();

        let mut bvh = Self::default();
        let mut root = Node::default();
        for tri in &scene.tris {
            root.grow_by_tri(tri);
        }
        root.num_tris = scene.tris.len() as u32;
        bvh.nodes.push(root);

        Self::split_node(0, &mut bvh, scene);

        let mut leaf_node_count: u32 = 0;
        let mut avg_tri_count: f32 = 0.0;
        let mut min_tri_count: u32 = u32::MAX;
        let mut max_tri_count: u32 = 0;
        bvh.nodes.iter().for_each(|node| {
            if node.num_tris != 0 {
                leaf_node_count += 1;
                if node.num_tris > max_tri_count {
                    max_tri_count = node.num_tris;
                } else if node.num_tris < min_tri_count {
                    min_tri_count = node.num_tris;
                }
                avg_tri_count += node.num_tris as f32;
            }
        });
        avg_tri_count /= leaf_node_count as f32;

        log_info!("BVH statistics");
        log_info!("- Build time:    {} ms", start_time.elapsed().as_millis());
        log_info!("- Total nodes:   {}", bvh.nodes.len());
        log_info!("- Leaf nodes:    {}", leaf_node_count);
        log_info!("- Avg leaf tris: {}", avg_tri_count);
        log_info!("- Min leaf tris: {}", min_tri_count);
        log_info!("- Max leaf tris: {}\n", max_tri_count);

        scene.bvh = bvh;
    }

    fn split_node(index: usize, bvh: &mut Self, scene: &mut Scene) {
        let used_nodes = bvh.nodes.len() as u32;
        let node = bvh.nodes.get_mut(index).unwrap();

        let parent_cost = node.num_tris as f32 * node.surface_area();

        // Surface area heuristic
        const NUM_BINS: usize = 8;
        let mut best_split_axis: usize = 0;
        let mut best_split_pos: f32 = 0.0;
        let mut best_split_cost: f32 = f32::MAX;

        let mut centroid_extent = (Vec3f::from(f32::MAX), Vec3f::from(f32::MIN));
        for i in 0..node.num_tris {
            let tri = scene.tris[(node.first_tri_or_child + i) as usize];
            let tri_bounds_mid = tri.bounds_mid();
            centroid_extent.0 = Vec3f::min(centroid_extent.0, tri_bounds_mid);
            centroid_extent.1 = Vec3f::max(centroid_extent.1, tri_bounds_mid);
        }

        for split_axis in 0..3 {
            if centroid_extent.0.data[split_axis] == centroid_extent.1.data[split_axis] {
                continue;
            }

            let scale = (centroid_extent.1.data[split_axis] - centroid_extent.0.data[split_axis])
                / NUM_BINS as f32;
            for i in 0..NUM_BINS {
                let split_pos = centroid_extent.0.data[split_axis] + i as f32 * scale;
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

        // Sort triangles
        let mut i: u32 = node.first_tri_or_child;
        let mut j: u32 = i + node.num_tris - 1;
        while i <= j {
            if scene.tris[i as usize].bounds_mid().data[best_split_axis] < best_split_pos {
                i += 1;
            } else {
                scene.tris.swap(i as usize, j as usize);
                j -= 1;
            }
        }

        let left_count = i - node.first_tri_or_child;
        if left_count == 0 || left_count == node.num_tris {
            return;
        }

        let mut left = Node::default();
        let mut right = Node::default();
        left.first_tri_or_child = node.first_tri_or_child;
        left.num_tris = left_count;
        right.first_tri_or_child = i;
        right.num_tris = node.num_tris - left_count;
        node.first_tri_or_child = used_nodes;
        node.num_tris = 0;

        for i in 0..left.num_tris {
            left.grow_by_tri(&scene.tris[(left.first_tri_or_child + i) as usize]);
        }
        for i in 0..right.num_tris {
            right.grow_by_tri(&scene.tris[(right.first_tri_or_child + i) as usize]);
        }

        bvh.nodes.push(left);
        bvh.nodes.push(right);

        Self::split_node(used_nodes as usize, bvh, scene);
        Self::split_node((used_nodes + 1) as usize, bvh, scene);
    }

    fn evaluate_sah(scene: &Scene, node: &Node, split_axis: usize, split_pos: f32) -> f32 {
        let mut left = Node::default();
        let mut right = Node::default();

        for i in 0..node.num_tris {
            let tri = &scene.tris[(node.first_tri_or_child + i) as usize];
            if tri.bounds_mid().data[split_axis] < split_pos {
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

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Node {
    pub bounds_min: Vec3f,
    pub first_tri_or_child: u32,
    pub bounds_max: Vec3f,
    pub num_tris: u32,
}

impl Default for Node {
    fn default() -> Self {
        return Self {
            bounds_min: Vec3f::from(f32::MAX),
            first_tri_or_child: 0,
            bounds_max: Vec3f::from(-f32::MAX),
            num_tris: 0,
        };
    }
}

impl AABB for Node {
    fn bounds(&self) -> (Vec3f, Vec3f) {
        (self.bounds_min, self.bounds_max)
    }

    fn bounds_mut(&mut self) -> (&mut Vec3f, &mut Vec3f) {
        (&mut self.bounds_min, &mut self.bounds_max)
    }
}

impl AABB for (Vec3f, Vec3f) {
    fn bounds(&self) -> (Vec3f, Vec3f) {
        (self.0, self.1)
    }

    fn bounds_mut(&mut self) -> (&mut Vec3f, &mut Vec3f) {
        (&mut self.0, &mut self.1)
    }
}

trait AABB {
    fn bounds(&self) -> (Vec3f, Vec3f);

    fn bounds_mut(&mut self) -> (&mut Vec3f, &mut Vec3f);

    fn grow_by_tri(&mut self, tri: &Triangle) {
        for vertex in tri.vertices {
            *self.bounds_mut().0 = Vec3f::min(self.bounds().0, vertex.position);
            *self.bounds_mut().1 = Vec3f::max(self.bounds().1, vertex.position);
        }
    }

    fn grow_by_aabb<T>(&mut self, other: T)
    where
        T: AABB,
    {
        *self.bounds_mut().0 = Vec3f::min(self.bounds().0, other.bounds().0);
        *self.bounds_mut().1 = Vec3f::max(self.bounds().1, other.bounds().1);
    }

    fn extent(&self) -> Vec3f {
        self.bounds().1 - self.bounds().0
    }

    fn surface_area(&self) -> f32 {
        let extent = self.extent();
        return (extent.x() * extent.z()) + (extent.x() * extent.y()) + (extent.z() * extent.y());
    }
}
