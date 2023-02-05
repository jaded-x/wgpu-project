use std::ops::Deref;

use egui::epaint::ClippedPrimitive;
use egui_wgpu::renderer::ScreenDescriptor;
use specs::{World, WorldExt, Join};
use wgpu::{Surface, Device, Queue};

use super::{
    texture, 
    components::{mesh::{Mesh, Vert}, 
    transform::Transform, renderable::Renderable}, 
    context::{create_render_pipeline, Context}
};

pub struct Renderer {
    pub clear_color: wgpu::Color,
    pub texture_view: wgpu::TextureView,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let clear_color = wgpu::Color::BLACK;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mandelbrot Texture"),
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                None,
                &[Vert::desc()],
                shader,
            )
        };

        Self {
            clear_color,
            texture_view,
            transform_bind_group_layout,
            render_pipeline,
        }
    }
}

pub trait Pass {
    fn draw(&mut self, surface: &Surface, device: &Device, queue: &Queue, world: &World) -> Result<(), wgpu::SurfaceError>;
}

impl Pass for Renderer {
    fn draw(&mut self, surface: &Surface, device: &Device, queue: &Queue, world: &World) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let meshes = world.read_storage::<Mesh>();
        let transforms = world.read_storage::<Transform>();
        let mut renderables = world.write_storage::<Renderable>();

        for (transform, renderable) in (&transforms, &mut renderables).join()  {
            renderable.update_buffer(&queue, transform.clone());
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        for (renderable, mesh) in (&renderables, &meshes).join()  {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &renderable.bind_group, &[]);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
        
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}