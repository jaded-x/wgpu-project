struct Transform {
    matrix: mat4x4<f32>,
    ti_matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;


struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
};
@group(3) @binding(0)
var<storage, read> lights: array<PointLight>;
@group(3) @binding(1)
var<uniform> light_count: i32;

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
    @location(2) tangent_view_position: vec3<f32>,
    @location(3) tangent_matrix_0: vec3<f32>,
    @location(4) tangent_matrix_1: vec3<f32>,
    @location(5) tangent_matrix_2: vec3<f32>,
};

@vertex
fn vs_main (
    model: VertexInput,
) -> VertexOutput {
    let normal_matrix = mat3x3<f32>(
        transform.ti_matrix[0][0], transform.ti_matrix[0][1], transform.ti_matrix[0][2],
        transform.ti_matrix[1][0], transform.ti_matrix[1][1], transform.ti_matrix[1][2],
        transform.ti_matrix[2][0], transform.ti_matrix[2][1], transform.ti_matrix[2][2],
    );

    let world_normal = normalize(normal_matrix * model.normal);
    var world_tangent = normalize(normal_matrix * model.tangent);
    world_tangent = normalize(world_tangent - dot(world_tangent, world_normal) * world_normal);
    let world_bitangent = cross(world_normal, world_tangent);

    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position: vec4<f32> = transform.matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_matrix_0 = vec3<f32>(tangent_matrix[0]);
    out.tangent_matrix_1 = vec3<f32>(tangent_matrix[1]);
    out.tangent_matrix_2 = vec3<f32>(tangent_matrix[2]);

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
    let tangent_matrix = mat3x3<f32>(in.tangent_matrix_0, in.tangent_matrix_1, in.tangent_matrix_2);

    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(diffuse_color, 1.0);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    let tangent_normal = normalize(object_normal.xyz * 2.0 - 1.0);
    var light_dir = normalize((tangent_matrix * lights[0].position) - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);

    var result = calculate_point_light(lights[0], in.tangent_position, tangent_normal, view_dir, light_dir);

    for (var i = 1; i < light_count; i = i + 1) {
        var light_dir = normalize((tangent_matrix * lights[i].position) - in.tangent_position);
        result += calculate_point_light(lights[i], in.tangent_position, tangent_normal, view_dir, light_dir);
    }

    result *= object_color.xyz;

    let gamma = 2.2;

    return vec4<f32>(pow(result, vec3(1.0 / gamma)), object_color.a);
}

fn calculate_point_light(light: PointLight, tangent_position: vec3<f32>, tangent_normal: vec3<f32>, view_dir: vec3<f32>, light_dir: vec3<f32>) -> vec3<f32>{
    let distance = length(light.position - tangent_position);
    let attenuation = 1.0 / distance;

    let ambient_strength = 0.005;
    let ambient_color = light.color * ambient_strength * attenuation;

    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength * attenuation;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color * attenuation;

    return (ambient_color + diffuse_color + specular_color);
}