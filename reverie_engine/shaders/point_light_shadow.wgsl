struct Projection {
    proj_view: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> projection: Projection;

struct Transform {
    matrix: mat4x4<f32>,
    ti_matrix: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> transform: Transform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_position: vec3<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    output.v_position = input.position;
    output.position = projection.proj_view * (transform.matrix * vec4<f32>(input.position, 1.0));
    return output;
}

@fragment
fn fs_main() {}