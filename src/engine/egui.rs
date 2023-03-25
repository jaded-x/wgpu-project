use egui_inspector::EguiInspect;
use specs::{*, WorldExt};

use super::context::Context;
use super::components::transform::Transform;

pub struct Egui {
    pub renderer: egui_wgpu::renderer::Renderer,
    pub state: egui_winit::State,
    pub context: egui::Context,
}

impl Egui {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>, context: &Context) -> Self {
        Self {
            renderer: egui_wgpu::renderer::Renderer::new(&context.device, context.config.format, None, 1),
            state: egui_winit::State::new(&event_loop),
            context: egui::Context::default()
        }
    }

    pub fn world_inspect(&self, egui_input: egui::RawInput, world: &specs::World) -> egui::FullOutput {
        self.context.run(egui_input, |context| {
            let style: egui::Style = (*context.style()).clone();
            context.set_style(style);

            let frame = egui::containers::Frame {
                fill: context.style().visuals.window_fill(),
                inner_margin: 10.0.into(),
                rounding: 5.0.into(),
                stroke: context.style().visuals.widgets.noninteractive.fg_stroke,
                ..Default::default()
            };

            egui::Window::new("Transform") 
                .resizable(true)
                .constrain(true)
                .frame(frame)
                .show(&context, |ui| {
                    let mut transforms = world.write_storage::<Transform>();
                    for transform in (&mut transforms).join() {
                        for node in transform.inspect(ui) {
                            if node.dragged() { transform.update_matrix(); }
                        }
                    }
                });
            
        })
    }

    pub fn render(&mut self, context: &Context, world: &mut World, window: &winit::window::Window, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let egui_input = self.state.take_egui_input(window);
        let egui_output = self.world_inspect(egui_input, world);
        
        let clipped_primitives = self.context.tessellate(egui_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: egui_winit::native_pixels_per_point(window)
        };

        for (id, image) in egui_output.textures_delta.set {
            self.renderer.update_texture(&context.device, &context.queue, id, &image);
        }

        self.renderer.update_buffers(&context.device, &context.queue, encoder, &clipped_primitives, &screen_descriptor);

        {
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

            self.renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in egui_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }
}
