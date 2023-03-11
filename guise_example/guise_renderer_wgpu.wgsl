struct TransformUniforms {
    matrix: mat4x4<f32>,
};

struct VertexOutput {
    @builtin(position) position:  vec4<f32>,
    @location(0)       tex_coord: vec2<f32>,
    @location(1)       color:     vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_transform: TransformUniforms;
@group(1) @binding(0) var          u_texture:   texture_2d<f32>;
@group(1) @binding(1) var          u_sampler:   sampler;

@vertex
fn vs_main(
    @location(0) in_position:  vec2<f32>,
    @location(1) in_tex_coord: vec2<f32>,
    @location(2) in_color:     u32,
) -> VertexOutput {
    var out: VertexOutput;

    // TODO(yan): Hex literal would be nice, but I couldn't find a way to
    // specify those in naga 0.6.0 and wgpu 0.10.1.
    let mask: u32 = 255u; // 0xff;

    out.position  = u_transform.matrix * vec4<f32>(in_position, 0.0, 1.0);
    out.tex_coord = in_tex_coord;
    out.color     = vec4<f32>(
        f32((in_color >> 24u) & mask) / 255.0,
        f32((in_color >> 16u) & mask) / 255.0,
        f32((in_color >> 8u)  & mask) / 255.0,
        f32((in_color >> 0u)  & mask) / 255.0,
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * textureSample(u_texture, u_sampler, in.tex_coord);
}
