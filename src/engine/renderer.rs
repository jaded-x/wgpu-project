use std::{rc::Rc, num::NonZeroU32};

use specs::prelude::*;

use super::{
    components::{
        mesh::Mesh, 
        material::MaterialComponent, transform::Transform,
    },
    context::{create_render_pipeline, Context}, 
    camera::Camera,
    texture::{Texture, self}, model::{Model, DrawModel, Vertex, ModelVertex, Material}, gpu::Gpu, light_manager::LightManager,
};

pub struct Renderer {
    pub clear_color: wgpu::Color,
    pub texture_view: wgpu::TextureView,
    pub depth_texture: Texture,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub material_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let clear_color = wgpu::Color::BLACK;

        let (texture_view, depth_texture) = create_depth_texture(device, config);

        let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        });

        let material_bind_group_layout = Rc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<cg::Vector3<f32>>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("material_bind_group_layout"),
        }));

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility:wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: Some(NonZeroU32::new(2).unwrap()),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility:wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
            label: Some("light_bind_group_layout")
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &transform_bind_group_layout,
                &camera_bind_group_layout,
                &material_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(wgpu::TextureFormat::Depth32Float),
                &[ModelVertex::desc()],
                shader,
            )
        };

        Self {
            clear_color,
            texture_view,
            depth_texture,
            transform_bind_group_layout,
            material_bind_group_layout,
            camera_bind_group_layout,
            light_bind_group_layout,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        (self.texture_view, self.depth_texture) = create_depth_texture(device, config);
    }
}

pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> (wgpu::TextureView, texture::Texture){
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

    (texture_view, depth_texture)
}

pub trait Pass {
    fn draw(&mut self, context: &Context, view: &wgpu::TextureView, world: &mut World, camera: &Camera, models: &Vec<Model>, materials: &mut Vec<Gpu<Material>>, lights: &LightManager) -> Result<(), wgpu::SurfaceError>;
}

impl Pass for Renderer {
    fn draw(&mut self, context: &Context, view: &wgpu::TextureView, world: &mut World, camera: &Camera, models: &Vec<Model>, materials: &mut Vec<Gpu<Material>>, lights: &LightManager) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
        });

        {
            let meshes = world.read_storage::<Mesh>();
            let transforms = world.read_storage::<Transform>();
            let materials_c = world.read_storage::<MaterialComponent>();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &camera.bind_group, &[]);
            render_pass.set_bind_group(3, &lights.bind_group, &[]);
            
            for (mesh, transform, material) in (&meshes, &transforms, &materials_c).join()  {
                render_pass.set_bind_group(0, &transform.bind_group, &[]);
                render_pass.draw_mesh(&models[mesh.mesh_id].meshes[0], &materials[material.material_id]);
            }
        }

        context.queue.submit([encoder.finish()]);
        Ok(())
    }
}