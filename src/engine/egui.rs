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
}

pub trait EguiInspect {
    fn ui(&mut self, ui: &mut egui::Ui, range: core::ops::RangeInclusive<f32>);
}

impl EguiInspect for Transform {
    fn ui(&mut self, ui: &mut egui::Ui, range: core::ops::RangeInclusive<f32>) {
        ui.add(egui::Slider::new(&mut self.position.x, range.clone()).text("x"));
        ui.add(egui::Slider::new(&mut self.position.y, range.clone()).text("y"));
        ui.add(egui::Slider::new(&mut self.position.z, range.clone()).text("z"));
    }
}