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
    return textureSample(in_texture, in_sampler, fs_input.tex_coord);
}
