struct Transform {
    matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;

@group(3) @binding(0)
var<uniform> light_position: vec3<f32>;
@group(3) @binding(1)
var<uniform> light_color: vec3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_light_position: vec3<f32>,
    @location(3) tangent_view_position: vec3<f32>,
};

@vertex
fn vs_main (
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * transform.matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;

    let world_normal = normalize(transform.normal_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    let world_tangent = normalize(transform.normal_matrix * vec4<f32>(model.tangent, 0.0)).xyz;
    let world_bitangent = normalize(transform.normal_matrix * vec4<f32>(model.bitangent, 0.0)).xyz;
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position = transform.matrix * vec4<f32>(model.position, 1.0);

    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_light_position = tangent_matrix * light_position;

    return out;
}

@group(2) @binding(0)
var<uniform> diffuse_color: vec3<f32>;
@group(2) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(2)
var s_diffuse: sampler;
@group(2) @binding(3)
var t_normal: texture_2d<f32>;
@group(2) @binding(4)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(diffuse_color, 1.0);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    let ambient_strength = 0.1;
    let ambient_color = light_color * ambient_strength;

    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir = normalize(in.tangent_light_position - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light_color * diffuse_strength;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light_color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}