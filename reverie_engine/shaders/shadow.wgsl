@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

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

@vertex
fn vs_main(
    input: VertexInput,
) -> @builtin(position) vec4<f32> {
    return projection * transform.matrix * vec4(input.position, 1.0);
}