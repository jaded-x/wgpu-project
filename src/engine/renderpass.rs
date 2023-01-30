use std::collections::HashMap;

use specs::{World, WorldExt, Join};
use wgpu::{Surface, Device, Queue, util::DeviceExt};

use crate::util::cast_slice;

use super::{texture, components::{mesh::{Mesh, Vert}, transform::Transform}, context::create_render_pipeline, uniform_pool::UniformPool};

pub struct RenderPass {
    pub depth_texture: texture::Texture,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
}

impl RenderPass {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

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

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &transform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/plane.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[Vert::desc()],
                shader,
            )
        };

        Self {
            depth_texture,
            transform_bind_group_layout,
            render_pipeline,
            bind_groups: vec![],
        }
    }
}

pub trait Pass {
    fn draw(&mut self, surface: &Surface, device: &Device, queue: &Queue, world: &World) -> Result<(), wgpu::SurfaceError>;
}

impl Pass for RenderPass {
    fn draw(&mut self, surface: &Surface, device: &Device, queue: &Queue, world: &World) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
    
        let meshes = world.read_storage::<Mesh>();
        let transforms = world.read_storage::<Transform>();

        for transform in transforms.join()  {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: cast_slice(&[transform.aligned()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.transform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding()
                    }
                ],
                label: None,
            });

            self.bind_groups.push(bind_group);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {r: 0.0, g: 0.0, b: 0.0, a: 1.0}),
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
            })
        });

        render_pass.set_pipeline(&self.render_pipeline);

        for (i, mesh) in meshes.join().enumerate()  {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.bind_groups[i], &[]);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
        
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}