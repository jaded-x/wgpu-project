@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> pos: vec3<f32>; 

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
    @builtin(position) clip_position: vec4<f32>,
    @location(0) v: vec3<f32>
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var position = projection * transform.matrix * vec4(input.position, 1.0);

    out.clip_position = position;
    out.v = position.xyz;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // var light_distance = length(in.v - pos);
    // light_distance = (light_distance - 0.1) / (15.0 - 0.1);

    //return vec4(light_distance, light_distance, light_distance, 1.0);

    let depth = in.clip_position.z / in.clip_position.w;
    //let linearDepth = (in.clip_position.z / in.clip_position.w - 0.1) / (15.0 - 0.1);
    return vec4(depth, depth, depth, 1.0);
}