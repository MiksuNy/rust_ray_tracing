struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VSOutput {
    let vertex_array = array(
        vec2<f32>(-1.0,  3.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0, -1.0),
    );

    var vs_output: VSOutput;
    let xy = vertex_array[in_vertex_index];
    vs_output.position = vec4<f32>(xy, 0.0, 1.0);
    vs_output.tex_coord = vec2<f32>(xy.x * 0.5 + 0.5, 1.0 - (xy.y * 0.5 + 0.5));
    return vs_output;
}

@group(0) @binding(0) var in_sampler: sampler;
@group(0) @binding(1) var in_texture: texture_2d<f32>;

@fragment
fn fs_main(fs_input: VSOutput) -> @location(0) vec4<f32> {
    var color = textureSample(in_texture, in_sampler, fs_input.tex_coord).rgb;
    color = linear_to_srgb(color);
    color = aces_filmic(color);
    return vec4<f32>(color, 1.0f);
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
