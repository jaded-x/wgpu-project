struct Transform {
    position: vec3<f32>,
    scale: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    vertices: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertices.position * transform.scale + transform.position, 1.0);
    out.color = vec3<f32>(0.0, 1.0, 0.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}