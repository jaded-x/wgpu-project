@group(0) @binding(2)
var<uniform> view_proj: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = in.position;
    out.position = view_proj * vec4<f32>(in.position, 1.0);

    return out;
}

@group(0) @binding(0)
var t_skybox: texture_cube<f32>;
@group(0) @binding(1)
var s_skybox: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return textureSample(t_skybox, s_skybox, in.tex_coords);
}