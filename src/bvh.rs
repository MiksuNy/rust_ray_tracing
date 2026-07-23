use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    log_error, log_info, log_warning,
    math::{
        vec::{Max, Min},
        vec3::*,
    },
    scene::{Scene, Triangle},
};

const SPLIT_FACTOR: f32 = 0.8;
const TRIANGLE_COST: f32 = 1.1;
const TRAVERSAL_COST: f32 = 1.0;

#[derive(Clone, Default)]
pub struct BVH {
    pub nodes: Vec<Node>,
    fragments: (Vec<Bounds>, Vec<usize>),
    fragment_ids_sorted_on_axis: [Vec<usize>; 3],
    right_costs: Vec<f32>,
    partition_left: Vec<bool>,
}

impl BVH {
    pub fn build(scene: &mut Scene) {
        let mut bvh = Self::default();

        log_info!("Building BVH for scene");

        let start_time = std::time::Instant::now();

        // Pre-split triangles
        bvh.fragments = Self::pre_split(&scene.tris, SPLIT_FACTOR);
        log_info!(
            "Pre-split created {} more triangles with a split factor of {}",
            bvh.fragments.0.len() - scene.tris.len(),
            SPLIT_FACTOR
        );

        // Allocate lists and sort primitives on each axis
        bvh.init_build_data();

        // Create root node
        let mut root = Node::default();
        for i in 0..bvh.fragments.0.len() {
            root.grow_by_aabb(&bvh.fragments.0[i]);
        }
        root.num_tris = bvh.fragments.0.len() as u32;
        bvh.nodes.push(root);

        // Recursively split root node
        Self::split_node(0, &mut bvh);

        let unindexed_tris = Self::get_unindexed_triangles(&mut bvh, &scene.tris);
        scene.tris = unindexed_tris;

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
        log_info!("- Build time:     {} ms", start_time.elapsed().as_millis());
        log_info!("- Total nodes:    {}", bvh.nodes.len());
        log_info!("- Leaf nodes:     {}", leaf_node_count);
        log_info!("- Avg leaf tris:  {}", avg_tri_count);
        log_info!("- Min leaf tris:  {}", min_tri_count);
        log_info!("- Max leaf tris:  {}", max_tri_count);
        log_info!("- Total SAH cost: {}\n", bvh.compute_global_sah());

        scene.bvh = bvh;
    }

    fn init_build_data(&mut self) {
        self.right_costs = vec![0.0f32; self.fragments.0.len()];
        self.partition_left = vec![false; self.fragments.0.len()];

        // Sort primitives on each axis
        for axis in 0..3 {
            self.fragment_ids_sorted_on_axis[axis] = (0..self.fragments.0.len()).collect();

            // par_sort_by is a bit faster here than sort_by
            self.fragment_ids_sorted_on_axis[axis].par_sort_by(|a, b| {
                let a_pos: f32 = self.fragments.0[*a].center().data[axis];
                let b_pos: f32 = self.fragments.0[*b].center().data[axis];
                return a_pos.partial_cmp(&b_pos).unwrap();
            });
        }
    }

    fn split_node(index: usize, bvh: &mut Self) {
        let used_nodes = bvh.nodes.len() as u32;
        let node = &mut bvh.nodes[index];

        // SweepSAH
        let mut best_split_cost: f32 = f32::MAX;
        let mut best_split_index: usize = 0;
        let mut best_split_axis: usize = 0;

        let start = node.first_tri_or_child as usize;
        let end = start + node.num_tris as usize;

        for split_axis in 0..3 {
            let indices = &bvh.fragment_ids_sorted_on_axis[split_axis];

            let mut right_box_accum = Bounds::default();
            let mut right_counter: usize = 0;
            let mut i: usize = end - 1;
            while i >= start + 1 {
                right_counter += 1;
                right_box_accum.grow_by_aabb(&bvh.fragments.0[indices[i]]);

                bvh.right_costs[i] = right_box_accum.half_area() * right_counter as f32;

                i -= 1;
            }

            let mut left_box_accum = Bounds::default();
            let mut left_counter: usize = 0;
            let mut i: usize = start;
            while i < end - 1 {
                left_counter += 1;
                left_box_accum.grow_by_aabb(&bvh.fragments.0[indices[i]]);

                let left_cost = left_box_accum.half_area() * left_counter as f32;
                let right_cost = bvh.right_costs[i + 1];
                let cost = left_cost + right_cost;

                if cost < best_split_cost {
                    best_split_cost = cost;
                    best_split_index = i + 1;
                    best_split_axis = split_axis;
                } else if left_cost >= best_split_cost {
                    break;
                }

                i += 1;
            }
        }
        best_split_cost = TRAVERSAL_COST + (TRIANGLE_COST * best_split_cost / node.half_area());

        // Partition primitives
        for i in start..best_split_index {
            bvh.partition_left[bvh.fragment_ids_sorted_on_axis[best_split_axis][i]] = true;
        }
        for i in best_split_index..end {
            bvh.partition_left[bvh.fragment_ids_sorted_on_axis[best_split_axis][i]] = false;
        }

        let mut partition: (Vec<usize>, Vec<usize>) = bvh.fragment_ids_sorted_on_axis
            [(best_split_axis + 1) % 3][start..end]
            .iter()
            .partition(|&i| bvh.partition_left[*i]);
        partition.0.append(&mut partition.1);
        bvh.fragment_ids_sorted_on_axis[(best_split_axis + 1) % 3][start..end]
            .swap_with_slice(&mut partition.0);

        partition = bvh.fragment_ids_sorted_on_axis[(best_split_axis + 2) % 3][start..end]
            .iter()
            .partition(|&i| bvh.partition_left[*i]);
        partition.0.append(&mut partition.1);
        bvh.fragment_ids_sorted_on_axis[(best_split_axis + 2) % 3][start..end]
            .swap_with_slice(&mut partition.0);

        let parent_cost = node.num_tris as f32 * TRIANGLE_COST;
        if best_split_cost >= parent_cost {
            return;
        }

        let mut left = Node::default();
        left.first_tri_or_child = node.first_tri_or_child;
        left.num_tris = best_split_index as u32 - left.first_tri_or_child;

        let mut right = Node::default();
        right.first_tri_or_child = best_split_index as u32;
        right.num_tris = node.num_tris - left.num_tris;

        node.first_tri_or_child = used_nodes;
        node.num_tris = 0;

        let indices = &bvh.fragment_ids_sorted_on_axis[best_split_axis];
        for i in 0..left.num_tris {
            left.grow_by_aabb(&bvh.fragments.0[indices[(left.first_tri_or_child + i) as usize]]);
        }
        for i in 0..right.num_tris {
            right.grow_by_aabb(&bvh.fragments.0[indices[(right.first_tri_or_child + i) as usize]]);
        }

        bvh.nodes.push(left);
        bvh.nodes.push(right);

        Self::split_node(used_nodes as usize, bvh);
        Self::split_node((used_nodes + 1) as usize, bvh);
    }

    // PreSplitting: https://github.com/BoyBaykiller/IDKEngine/tree/master#better-bvhs-with-presplitting
    fn pre_split(tris: &[Triangle], split_factor: f32) -> (Vec<Bounds>, Vec<usize>) {
        let mut total_priority: f32 = 0.0;
        for tri in tris {
            total_priority += Self::priority(tri);
        }

        let mut counter: usize = 0;
        for tri in tris {
            let priority = Self::priority(tri);
            let split_count =
                Self::get_split_count(priority, total_priority, tris.len(), split_factor);
            counter += split_count;
        }

        let mut bounds = vec![Bounds::default(); counter];
        let mut original_tri_ids = vec![0; counter];

        counter = 0;

        let mut global_bounds = Bounds::default();
        for i in 0..bounds.len() {
            global_bounds.grow_by_aabb(&bounds[i]);
        }
        let global_extent = global_bounds.extent();

        let mut stack = [(Bounds::default(), 0); 64];
        for i in 0..tris.len() {
            let tri = tris[i];

            let priority = Self::priority(&tri);
            let split_count =
                Self::get_split_count(priority, total_priority, tris.len(), split_factor);

            let mut stack_ptr: usize = 0;
            stack[stack_ptr] = (Bounds::from(tri), split_count);
            stack_ptr += 1;
            while stack_ptr > 0 {
                stack_ptr -= 1;
                let (parent_box, splits_left) = stack[stack_ptr];

                if splits_left == 1 {
                    bounds[counter] = parent_box;
                    original_tri_ids[counter] = i;
                    counter += 1;
                    continue;
                }

                let split_axis = parent_box.largest_axis();
                let largest_extent = parent_box.largest_extent();

                let mut node_size =
                    Self::get_node_size(largest_extent, global_extent.data[split_axis]);
                if node_size >= largest_extent - 0.0001 {
                    node_size *= 0.5;
                }

                let mid_pos =
                    (parent_box.min.data[split_axis] + parent_box.max.data[split_axis]) * 0.5;
                let index = f32::round((mid_pos - global_bounds.min.data[split_axis]) / node_size);
                let split_pos = global_bounds.min.data[split_axis] + index * node_size;

                let (mut left_box, mut right_box) = tri.split(split_axis, split_pos);
                left_box.clip_against_aabb(&parent_box);
                right_box.clip_against_aabb(&parent_box);

                let left_extent = left_box.largest_extent();
                let right_extent = right_box.largest_extent();

                let mut left_count =
                    (splits_left as f32 * (left_extent / (left_extent + right_extent))) as usize;
                left_count = usize::clamp(left_count, 1, splits_left - 1);

                let right_count = splits_left - left_count;

                stack[stack_ptr] = (right_box, right_count);
                stack_ptr += 1;
                stack[stack_ptr] = (left_box, left_count);
                stack_ptr += 1;
            }
        }

        return (bounds, original_tri_ids);
    }

    fn priority(tri: &Triangle) -> f32 {
        let tri_bounds: Bounds = Bounds::from(*tri);

        let largest_extent = tri_bounds.largest_extent();
        let extent_prio = largest_extent * largest_extent;
        let empty_area_prio = tri_bounds.surface_area() - tri.surface_area();

        return f32::cbrt(extent_prio * empty_area_prio);
    }

    fn get_split_count(
        priority: f32,
        total_priority: f32,
        tri_count: usize,
        split_factor: f32,
    ) -> usize {
        let share_of_tris = priority / total_priority * tri_count as f32;
        return 1 + (share_of_tris * split_factor) as usize;
    }

    fn get_node_size(extent: f32, global_size: f32) -> f32 {
        let alpha = extent / global_size;
        let exponent_bits: u32 = alpha.to_bits() & (255u32 << 23u32);
        return f32::from_bits(exponent_bits) * global_size;
    }

    fn get_unindexed_triangles(bvh: &mut Self, tris: &[Triangle]) -> Vec<Triangle> {
        let mut new_triangles: Vec<Triangle> = vec![Triangle::default(); bvh.fragments.1.len()];
        let mut tri_counter: usize = 0;

        for i in 0..bvh.nodes.len() {
            let node = &mut bvh.nodes[i];
            if node.num_tris != 0 {
                for j in 0..node.num_tris {
                    let fragment_id =
                        bvh.fragment_ids_sorted_on_axis[0][(node.first_tri_or_child + j) as usize];
                    let orig_tri_id = bvh.fragments.1[fragment_id];
                    new_triangles[(tri_counter as u32 + j) as usize] = tris[orig_tri_id];
                }

                node.first_tri_or_child = tri_counter as u32;
                tri_counter += node.num_tris as usize;
            }
        }

        return new_triangles;
    }

    fn compute_global_sah(&self) -> f32 {
        let root_area = self.nodes[0].half_area();

        let mut total_cost = 0.0;
        for node in &self.nodes {
            let prob_hit_node = node.half_area() / root_area;
            if node.num_tris != 0 {
                total_cost += TRIANGLE_COST * node.num_tris as f32 * prob_hit_node;
            } else {
                total_cost += TRAVERSAL_COST * prob_hit_node;
            }
        }

        return total_cost;
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
            bounds_max: Vec3f::from(f32::MIN),
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

#[derive(Clone, Copy)]
pub struct Bounds {
    min: Vec3f,
    max: Vec3f,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Vec3f::from(f32::MAX),
            max: Vec3f::from(f32::MIN),
        }
    }
}

impl AABB for Bounds {
    fn bounds(&self) -> (Vec3f, Vec3f) {
        (self.min, self.max)
    }

    fn bounds_mut(&mut self) -> (&mut Vec3f, &mut Vec3f) {
        (&mut self.min, &mut self.max)
    }
}

impl From<Triangle> for Bounds {
    fn from(tri: Triangle) -> Self {
        let mut bounds = Bounds::default();
        bounds.grow_by_tri(&tri);
        return bounds;
    }
}

pub trait AABB {
    fn bounds(&self) -> (Vec3f, Vec3f);

    fn bounds_mut(&mut self) -> (&mut Vec3f, &mut Vec3f);

    fn grow_by_tri(&mut self, tri: &Triangle) {
        for vertex in tri.vertices {
            *self.bounds_mut().0 = Vec3f::min(self.bounds().0, vertex.position);
            *self.bounds_mut().1 = Vec3f::max(self.bounds().1, vertex.position);
        }
    }

    fn grow_by_aabb<T>(&mut self, other: &T)
    where
        T: AABB,
    {
        *self.bounds_mut().0 = Vec3f::min(self.bounds().0, other.bounds().0);
        *self.bounds_mut().1 = Vec3f::max(self.bounds().1, other.bounds().1);
    }

    fn grow_by_position(&mut self, pos: Vec3f) {
        *self.bounds_mut().0 = Vec3f::min(self.bounds().0, pos);
        *self.bounds_mut().1 = Vec3f::max(self.bounds().1, pos);
    }

    fn clip_against_aabb<T>(&mut self, other: &T)
    where
        T: AABB,
    {
        *self.bounds_mut().0 = Vec3f::max(self.bounds().0, other.bounds().0);
        *self.bounds_mut().1 = Vec3f::min(self.bounds().1, other.bounds().1);
    }

    fn extent(&self) -> Vec3f {
        self.bounds().1 - self.bounds().0
    }

    fn largest_extent(&self) -> f32 {
        let extent = self.extent();
        let mut largest: f32 = 0.0;
        for axis in extent.data {
            largest = f32::max(largest, axis);
        }
        return largest;
    }

    fn largest_axis(&self) -> usize {
        let extent = self.extent();
        let mut axis = 0;
        if extent.data[0] < extent.data[1] {
            axis = 1;
        }
        if extent.data[axis] < extent.data[2] {
            axis = 2;
        }
        return axis;
    }

    fn surface_area(&self) -> f32 {
        let extent = self.extent();
        return ((extent.x() * extent.z()) + (extent.x() * extent.y()) + (extent.z() * extent.y()))
            * 2.0;
    }

    fn half_area(&self) -> f32 {
        let extent = self.extent();
        return (extent.x() * extent.z()) + (extent.x() * extent.y()) + (extent.z() * extent.y());
    }

    fn center(&self) -> Vec3f {
        (self.bounds().0 + self.bounds().1) / 2.0
    }
}
