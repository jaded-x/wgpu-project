use egui_inspector::{EguiInspect};
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

            egui::Window::new("Position") 
                .resizable(true)
                .constrain(true)
                .frame(frame)
                .show(&context, |ui| {
                    let mut transforms = world.write_storage::<Transform>();
                    for transform in (&mut transforms).join() {
                        transform.inspect(ui);
                    }
                });
        })
    }
}

// pub trait EguiInspect {
//     fn ui(&mut self, ui: &mut egui::Ui, range: core::ops::RangeInclusive<f32>, scale_range: core::ops::RangeInclusive<f32>);
// }

// impl EguiInspect for Transform {
//     fn ui(&mut self, ui: &mut egui::Ui, range: core::ops::RangeInclusive<f32>, scale_range: core::ops::RangeInclusive<f32>) {
//         ui.add(egui::DragValue::new(&mut self.position.x).speed(0.02));
//         ui.add(egui::Slider::new(&mut self.position.y, range.clone()).text("y"));
//         ui.add(egui::Slider::new(&mut self.position.z, range.clone()).text("z"));
//         ui.add(egui::Slider::new(&mut self.scale.x, scale_range.clone()).text("x"));
//         ui.add(egui::Slider::new(&mut self.scale.y, scale_range.clone()).text("y"));
//         ui.add(egui::Slider::new(&mut self.scale.z, scale_range.clone()).text("z"));
//     }
// }