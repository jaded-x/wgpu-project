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

struct Material {
    color: vec3<f32>
}
@group(2) @binding(0)
var<uniform> material: Material;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(4) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main (
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = camera.view_proj * transform.matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = material.color;

    return out;
}

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(in.color, 1.0);
    
    let ambient_strength = 0.1;
    let ambient_color = 
    
    return color;
}