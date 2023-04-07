use egui_inspector::EguiInspect;
use specs::{*, WorldExt};

use crate::util::cast_slice;

use super::model::Material;
use super::render::Render;
use super::{
    context::Context,
};
use super::components::{
    transform::Transform,
    material::MaterialComponent,
};

pub struct Egui {
    pub renderer: egui_wgpu::renderer::Renderer,
    pub state: egui_winit::State,
    pub context: egui::Context,
    entity: Option<specs::Entity>,
    material_id: Option<usize>,
    material: Option<Material>,
}

impl Egui {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>, context: &Context) -> Self {
        Self {
            renderer: egui_wgpu::renderer::Renderer::new(&context.device, context.config.format, None, 1),
            state: egui_winit::State::new(&event_loop),
            context: egui::Context::default(),
            entity: None,
            material: None,
            material_id: None,
        }
    }

    pub fn world_inspect(&mut self, egui_input: egui::RawInput, world: &specs::World, materials: &mut Vec<Render<Material>>) -> egui::FullOutput {
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

            egui::Window::new("World")
                .resizable(true)
                .constrain(true)
                .show(&context, |ui| {
                    for entity in world.entities().join() {
                        let res = ui.add(egui::Button::new(entity.id().to_string()));
                        if res.clicked() { 
                            self.material = None;
                            self.material_id = None;
                            self.entity = Some(entity)
                        }
                    }
                });

            egui::Window::new("Materials")
                .resizable(true)
                .constrain(true)
                .show(&context, |ui| {
                    for (i, material) in materials.iter().enumerate() {
                        let res = ui.add(egui::Button::new(i.to_string()));
                        if res.clicked() { 
                            self.entity = None;
                            self.material_id = Some(i);
                            self.material = Some(material.asset.borrow().clone());
                        }
                    }
                });

            egui::Window::new("Inspector") 
                .resizable(true)
                .constrain(true)
                .frame(frame)
                .show(&context, |ui| {
                    if let Some(entity) = self.entity {
                        let mut transforms = world.write_storage::<Transform>();
                        let transform = transforms.get_mut(entity).unwrap();
                        let mut materials = world.write_storage::<MaterialComponent>();
                        let material = materials.get_mut(entity).unwrap();

                        ui.label(entity.id().to_string());
                        egui::CollapsingHeader::new("Transform")
                            .default_open(true)
                            .show(ui, |ui| {
                                for field in transform.inspect(ui) {
                                    if field.changed() { transform.update_matrix(); }
                                }
                            });
                        egui::CollapsingHeader::new("Material")
                            .default_open(true)
                            .show(ui, |ui| {
                                material.inspect(ui);
                            });
                    } else if let Some(material) = &mut self.material {
                        for field in material.inspect(ui) {
                            if field.changed() { materials[self.material_id.unwrap()].update_buffer(0, cast_slice(&[material.diffuse]))}
                        }
                    }
                });
        })
    }

    pub fn render(&mut self, context: &Context, world: &mut World, materials: &mut Vec<Render<Material>>, window: &winit::window::Window, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let egui_input = self.state.take_egui_input(window);
        let egui_output = self.world_inspect(egui_input, world, materials);
        
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
