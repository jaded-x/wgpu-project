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
    @location(0) normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let world_position: vec4<f32> = transform.matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.position = camera.view_proj * world_position;
    out.normal = model.normal;
    out.world_position = world_position.xyz;

    return out;
}

struct Material {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
}
@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(2)
var s_diffuse: sampler;
@group(2) @binding(3)
var t_normal: texture_2d<f32>;
@group(2) @binding(4)
var s_normal: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(camera.view_pos.xyz - in.world_position);

    var f0 = vec3<f32>(0.04);
    f0 = mix(f0, material.albedo, material.metallic);

    var lo = vec3<f32>(0.0);
    for (var i = 0; i < light_count; i = i + 1) {
        let l = normalize(lights[i].position - in.world_position);
        let h = normalize(v + l);
        let distance = length(lights[i].position - in.world_position);
        let attenuation = 1.0 / (distance * distance);
        let radiance = lights[i].color * attenuation;

        let ndf = distributionggx(n, h, material.roughness);
        let g = geometrysmith(n, v, l, material.roughness);
        let f = fresnelschlick(clamp(dot(h, v), 0.0, 1.0), f0);

        let numerator = ndf * g * f;
        let denominator = 4.0 / max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let ks = f;
        var kd = vec3<f32>(1.0) - ks;
        kd = kd * (1.0 - material.metallic);
        let nl = max(dot(n, l), 0.0);
        lo = lo + ((kd * material.albedo / PI + specular) * radiance * nl);
    }

    let ambient = vec3<f32>(0.03) * material.albedo * material.ao;
    var color = ambient + lo;
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0/2.2));

    return vec4<f32>(color, 1.0);
}

const PI = 3.14159265359;

fn distributionggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let nh = max(dot(N, H), 0.0);
    let nh2 = nh * nh;

    var denom = (nh2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return a2 / denom;
}

fn geometryschlickggx(nv: f32, roughness: f32) -> f32{
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let denom = nv * (1.0 - k) + k;

    return nv / denom;
}

fn geometrysmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let nv = max(dot(N, V), 0.0);
    let nl = max(dot(N, L), 0.0);
    let ggx2 = geometryschlickggx(nv, roughness);
    let ggx1 = geometryschlickggx(nl, roughness);

    return ggx1 * ggx2;
}

fn fresnelschlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

