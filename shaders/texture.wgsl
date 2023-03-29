struct Transform {
    matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main (
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = camera.view_proj * transform.matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = vec4(in.tex_coords, 0.0, 1.0);
    return color;
}