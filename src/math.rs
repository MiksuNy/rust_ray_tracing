// https://www.reedbeta.com/blog/quick-and-easy-gpu-random-numbers-in-d3d11/
fn wang_hash(seed: &mut u32) -> u32 {
    *seed = (*seed ^ 61) ^ (*seed >> 16);
    *seed *= 9;
    *seed = *seed ^ (*seed >> 4);
    *seed *= 0x27d4eb2d;
    *seed = *seed ^ (*seed >> 15);
    return *seed;
}

fn rand_f32_nd(input: &mut u32) -> f32 {
    let theta = 6.283185 * rand_f32(input);
    let rho = f32::sqrt(-2.0 * f32::log10(rand_f32(input)));
    return rho * f32::cos(theta);
}

/// Returns a random f32 in the range 0.0 - 1.0
pub fn rand_f32(input: &mut u32) -> f32 {
    return wang_hash(input) as f32 / u32::MAX as f32;
}

pub fn rand_in_unit_sphere(input: &mut u32) -> glam::Vec3 {
    glam::Vec3::new(rand_f32_nd(input), rand_f32_nd(input), rand_f32_nd(input)).normalize()
}

pub fn rand_in_unit_hemisphere(input: &mut u32, normal: glam::Vec3) -> glam::Vec3 {
    let unit_sphere = rand_in_unit_sphere(input);
    if glam::Vec3::dot(unit_sphere, normal) < 0.0 {
        return -unit_sphere;
    } else {
        return unit_sphere;
    }
}

// https://gamedev.stackexchange.com/a/194038
pub fn linear_to_srgb(linear: glam::Vec3) -> glam::Vec3 {
    let cutoff = glam::Vec3::new(
        ((linear.x < 0.0031308) as u32) as f32,
        ((linear.y < 0.0031308) as u32) as f32,
        ((linear.z < 0.0031308) as u32) as f32,
    );
    let higher = glam::Vec3::new(1.055, 1.055, 1.055) * glam::Vec3::powf(linear, 1.0 / 2.4)
        - glam::Vec3::new(0.055, 0.055, 0.055);
    let lower = linear * glam::Vec3::new(12.92, 12.92, 12.92);
    return glam::Vec3::new(
        glam::FloatExt::lerp(higher.x, lower.x, cutoff.x),
        glam::FloatExt::lerp(higher.y, lower.y, cutoff.y),
        glam::FloatExt::lerp(higher.z, lower.z, cutoff.z),
    );
}

// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
pub fn aces_filmic(x: glam::Vec3) -> glam::Vec3 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return glam::Vec3::clamp(
        (x * (a * x + b)) / (x * (c * x + d) + e),
        glam::Vec3::new(0.0, 0.0, 0.0),
        glam::Vec3::new(1.0, 1.0, 1.0),
    );
}

pub fn color_to_bytes(color: glam::Vec3) -> [u8; 3] {
    [
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8,
    ]
}
