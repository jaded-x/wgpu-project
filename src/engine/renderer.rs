use legion::*;

use super::{
    components::{
        mesh::{Mesh, Vert}, 
        renderable::Renderable
    },
    context::{create_render_pipeline, Context}, egui::Egui
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
    fn draw(&mut self, context: &Context, world: &mut legion::World, window: &winit::window::Window, egui: &mut Egui) -> Result<(), wgpu::SurfaceError>;
}

impl Pass for Renderer {
    fn draw(&mut self, context: &Context, world: &mut legion::World, window: &winit::window::Window, egui: &mut Egui) -> Result<(), wgpu::SurfaceError> {
        let output = context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
        });

        let mut query = <(&Mesh, &Renderable)>::query();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    }
                })
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        for (mesh, renderable) in query.iter_mut(world)  {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &renderable.bind_group, &[]);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }

        drop(render_pass);

        // render egui
        let egui_input = egui.state.take_egui_input(window);
        let egui_output = egui.context.run(egui_input, |context| {
            let style: egui::Style = (*context.style()).clone();
            context.set_style(style);

            let frame = egui::containers::Frame::side_top_panel(&context.style());

            let mut test = 0;

            egui::SidePanel::left("top").frame(frame).show(&context, |ui| {
                ui.add(egui::Slider::new(&mut test, 0..=120).text("hi :)"));
            });
        });
        
        let clipped_primitives = egui.context.tessellate(egui_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: egui_winit::native_pixels_per_point(window)
        };

        for (id, image) in egui_output.textures_delta.set {
            egui.renderer.update_texture(&context.device, &context.queue, id, &image);
        }

        egui.renderer.update_buffers(&context.device, &context.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        egui.renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);

        drop(render_pass);

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for id in egui_output.textures_delta.free {
            egui.renderer.free_texture(&id);
        }

        Ok(())
    }
}