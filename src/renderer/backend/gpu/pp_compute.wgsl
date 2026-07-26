@group(0) @binding(0)
var rt_texture: texture_storage_2d<rgba16unorm, read_write>;

@group(1) @binding(0)
var pp_texture: texture_storage_2d<rgba16unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tex_coords = vec2<i32>(i32(global_id.x), i32(global_id.y));

    var color = textureLoad(rt_texture, tex_coords).rgb;
    color = linear_to_srgb(color);
    color = aces_filmic(color);

    textureStore(pp_texture, tex_coords, vec4<f32>(color, 1.0f));
}

// https://gamedev.stackexchange.com/a/194038
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(f32(linear.r < 0.0031308f), f32(linear.g < 0.0031308f), f32(linear.b < 0.0031308f));
    let higher = vec3<f32>(1.055) * pow(linear, vec3<f32>(1.0/2.4)) - vec3<f32>(0.055);
    let lower = linear * vec3<f32>(12.92);
    return mix(higher, lower, cutoff);
}

// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51f;
    let b = 0.03f;
    let c = 2.43f;
    let d = 0.59f;
    let e = 0.14f;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0f), vec3<f32>(1.0f));
}
