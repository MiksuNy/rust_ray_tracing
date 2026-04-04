pub mod mat4;
pub mod vec;
pub mod vec2;
pub mod vec3;

fn xor_shift(input: &mut u32) -> u32 {
    let mut x: u32 = *input;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *input = x;
    return x;
}

fn rand_f32_nd(input: &mut u32) -> f32 {
    let theta = 6.283185 * rand_f32(input);
    let rho = f32::sqrt(-2.0 * f32::log10(rand_f32(input)));
    return rho * f32::cos(theta);
}

/// Returns a random f32 in the range 0.0 - 1.0
pub fn rand_f32(input: &mut u32) -> f32 {
    return xor_shift(input) as f32 / u32::MAX as f32;
}
