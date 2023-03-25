use egui::Widget;
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
}