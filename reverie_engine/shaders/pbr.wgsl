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
    projection: array<mat4x4<f32>, 6>,
    position: vec3<f32>,
    color: vec3<f32>,
};
@group(3) @binding(0)
var<storage, read> point_lights: array<PointLight>;
@group(3) @binding(1)
var<uniform> point_light_count: i32;
struct DirectionalLight {
    direction: vec3<f32>,
    color: vec3<f32>,
};
@group(3) @binding(2)
var<storage, read> directional_lights: array<DirectionalLight>;
@group(3) @binding(3)
var<uniform> directional_light_count: i32;

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
    @location(2) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var world_position: vec4<f32> = transform.matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.position = camera.view_proj * world_position;
    out.normal = normalize((transform.ti_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    out.world_position = world_position.xyz;
    out.tex_coords = model.tex_coords;

    return out;
}

@group(2) @binding(0)
var t_albedo: texture_2d<f32>;
@group(2) @binding(1)
var s_albedo: sampler;
@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;
@group(2) @binding(4)
var t_metallic: texture_2d<f32>;
@group(2) @binding(5)
var s_metallic: sampler;
@group(2) @binding(6)
var t_roughness: texture_2d<f32>;
@group(2) @binding(7)
var s_roughness: sampler;
@group(2) @binding(8)
var t_ao: texture_2d<f32>;
@group(2) @binding(9)
var s_ao: sampler;

@group(3) @binding(4)
var t_depth_cube: texture_depth_cube_array;
@group(3) @binding(5)
var s_depth_cube: sampler;

struct PBR {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
}
@group(2) @binding(10)
var<uniform> pbr: PBR;

struct PBRBool {
    albedo: f32,
    metallic: f32,
    roughness: f32,
    ao: f32,
}
@group(2) @binding(11)
var<uniform> bools: PBRBool;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var albedo: vec3<f32>;
    var metallic: f32;
    var roughness: f32;
    var ao: f32;
    
    if (bools.albedo != 0.0) {
        albedo = pow(textureSample(t_albedo, s_albedo, in.tex_coords).rgb, vec3<f32>(2.2));
    } else {
        albedo = pbr.albedo;
    }
    
    if (bools.metallic != 0.0) {
        metallic = textureSample(t_metallic, s_metallic, in.tex_coords).r;
    } else {
        metallic = pbr.metallic;
    }
    
    if (bools.roughness != 0.0) {
        roughness = textureSample(t_roughness, s_roughness, in.tex_coords).r;
    } else {
        roughness = pbr.roughness;
    }
    
    if (bools.ao != 0.0) {
        ao = textureSample(t_ao, s_ao, in.tex_coords).r;
    } else {
        ao = pbr.ao;
    }

    let n = get_normal_from_map(in.normal, in.world_position, in.tex_coords);
    let v = normalize(camera.view_pos.xyz - in.world_position);

    var f0 = vec3<f32>(0.04);
    f0 = mix(f0, albedo, metallic);

    var lo = vec3<f32>(0.0);
    for (var i = 0; i < point_light_count; i += 1) {
        let l = normalize(point_lights[i].position - in.world_position);
        let h = normalize(v + l);
        
        let distance = length(point_lights[i].position - in.world_position);
        let attenuation = 1.0 / (distance * distance);
        let radiance = (point_lights[i].color * 255.0) * attenuation;

        let ndf = distributionggx(n, h, roughness);
        let g = geometrysmith(n, v, l, roughness);
        let f = fresnelschlick(max(dot(h, v), 0.0), f0);

        let numerator = ndf * g * f;
        let denominator = 4.0 / max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let ks = f;
        var kd = vec3<f32>(1.0) - ks;
        kd = kd * (1.0 - metallic);
        let nl = max(dot(n, l), 0.0);

        //let shadow_factor = textureSampleCompare(t_depth_cube, s_depth_cube, l, distance / 100.0);

        var face = get_cube_face(l);

        let fragment_pos_light_space = point_lights[i].projection[face] * vec4<f32>(in.world_position, 1.0);
        let depth = fragment_pos_light_space.z / fragment_pos_light_space.w;
        
        //let shadow = calculate_shadow(distance, depth, l, i);
        var shadow = 0.0;
        var closestDepth = textureSample(t_depth_cube, s_depth_cube, l, i);
        if (depth < closestDepth) {
            shadow = shadow + 1.0;
        }

        lo = lo + ((kd * albedo / PI + specular) * radiance * (nl * shadow));
    }

    // for (var i = 1; i < directional_light_count; i = i + 1) {
    //     let l = normalize(directional_lights[i].direction);
    //     let h = normalize(v + l);
    //     let radiance = directional_lights[i].color;

    //     let ndf = distributionggx(n, h, roughness);
    //     let g = geometrysmith(n, v, l, roughness);
    //     let f = fresnelschlick(max(dot(h, v), 0.0), f0);

    //     let numerator = ndf * g * f;
    //     let denominator = 4.0 / max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
    //     let specular = numerator / denominator;

    //     let ks = f;
    //     var kd = vec3<f32>(1.0) - ks;
    //     kd = kd * (1.0 - metallic);
    //     let nl = max(dot(n, l), 0.0);
    //     lo = lo + ((kd * albedo / PI + specular) * radiance * nl);
    // }

    let ambient = vec3<f32>(0.03) * albedo * ao;
    var color = ambient + lo;
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0/2.2));

    return vec4<f32>(color, 1.0);
}

const PI = 3.14159265359;

fn get_normal_from_map(normal: vec3<f32>, world_position: vec3<f32>, tex_coords: vec2<f32>) -> vec3<f32> {
    let tangent_normal: vec3<f32> = textureSample(t_normal, s_normal, tex_coords).xyz * 2.0 - 1.0;

    let q1 = dpdx(world_position);
    let q2 = dpdy(world_position);
    let st1 = dpdx(tex_coords);
    let st2 = dpdy(tex_coords);

    let n = normalize(normal);
    let t = normalize(q1 * st2.y - q2 * st1.y);
    let b = -normalize(cross(n, t));
    let tbn = mat3x3<f32>(t, b, n);

    return normalize(tbn * tangent_normal);
}

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

fn get_sample_offset_directions() -> array<vec3<f32>, 20> {
    return array<vec3<f32>, 20>(
        vec3<f32>( 1.0,  1.0,  1.0), vec3<f32>( 1.0, -1.0,  1.0), vec3<f32>(-1.0, -1.0,  1.0), vec3<f32>(-1.0,  1.0,  1.0), 
        vec3<f32>( 1.0,  1.0, -1.0), vec3<f32>( 1.0, -1.0, -1.0), vec3<f32>(-1.0, -1.0, -1.0), vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  0.0), vec3<f32>( 1.0, -1.0,  0.0), vec3<f32>(-1.0, -1.0,  0.0), vec3<f32>(-1.0,  1.0,  0.0),
        vec3<f32>( 1.0,  0.0,  1.0), vec3<f32>(-1.0,  0.0,  1.0), vec3<f32>( 1.0,  0.0, -1.0), vec3<f32>(-1.0,  0.0, -1.0),
        vec3<f32>( 0.0,  1.0,  1.0), vec3<f32>( 0.0, -1.0,  1.0), vec3<f32>( 0.0, -1.0, -1.0), vec3<f32>( 0.0,  1.0, -1.0)
    );
}

fn calculate_shadow(distance: f32, depth: f32, l: vec3<f32>, i: i32) -> f32 {
    var shadow = 0.0;
    let samples = 20;        
    let sample_directions = get_sample_offset_directions();
    let disk_radius = (1.0 + (distance / 100.0)) / 400.0;
    let bias = -0.001;

    var closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[0] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[1] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[2] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[3] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[4] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[5] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[6] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[7] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[8] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[9] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[10] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[11] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[12] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[13] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[14] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[15] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[16] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[17] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[18] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);
    closest_depth = textureSample(t_depth_cube, s_depth_cube, l + sample_directions[19] * disk_radius, i);
    shadow += compare_depth(depth, closest_depth, bias);

    return shadow / f32(samples);
}

fn compare_depth(depth: f32, closest_depth: f32, bias: f32) -> f32 {
    if (depth - bias< closest_depth) {
        return 1.0;
    }
    
    return 0.0;
}

fn get_cube_face(l: vec3<f32>) -> i32 {
    var face = 0;
    let absL = abs(l);
    if absL.x > absL.y && absL.x > absL.z {
        if l.x > 0.0 {
            face = 0;
        } else {
            face = 1;
        }
    } else if absL.y > absL.z {
        if l.y > 0.0 {
            face = 2;
        } else {
            face = 3;
        }
    } else {
        if l.z > 0.0 {
            face = 4;
        } else {
            face = 5;
        }
    }

    return face;
}