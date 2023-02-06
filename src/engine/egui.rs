use super::context::Context;

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