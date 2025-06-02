use crate::{obj::Model, vec3::Vec3};

pub struct BVH {
    nodes: Vec<AABB>,
}

struct AABB {
    bounds_min: Vec3,
    bounds_max: Vec3,
}

impl AABB {
    pub fn new(bounds_min: Vec3, bounds_max: Vec3) -> Self {
        return Self {
            bounds_min,
            bounds_max,
        };
    }
}
