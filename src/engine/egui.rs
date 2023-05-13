use egui_inspector::EguiInspect;
use specs::{*, WorldExt};

use super::light::PointLight;
use super::light_manager::LightManager;
use super::model::Material;
use super::gpu::Gpu;
use super::{
    context::Context,
};
use super::components::{
    transform::Transform,
    material::MaterialComponent,
    name::Name,
    mesh::Mesh,
};

pub struct Egui {
    pub renderer: egui_wgpu::renderer::Renderer,
    pub state: egui_winit::State,
    pub context: egui::Context,
    entity: Option<specs::Entity>,
    material_id: Option<usize>,
    material: Option<Material>,
    light_index: Option<u64>,
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
            light_index: None,
        }
    }

    pub fn world_inspect(&mut self, egui_input: egui::RawInput, world: &specs::World, light_manager: &LightManager, materials: &mut Vec<Gpu<Material>>, queue: &wgpu::Queue) -> egui::FullOutput {
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

            egui::SidePanel::right("Inspector")
                .default_width(200.0)
                .resizable(true)
                .frame(frame)
                .show(&context, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Inspector");
                        ui.separator();
                        if let Some(entity) = self.entity {
                            let mut names = world.write_component::<Name>();
                            let name = names.get_mut(entity).unwrap();
                            ui.text_edit_singleline(&mut name.0);
                            
                            let mut transforms = world.write_component::<Transform>();
                            if let Some(transform) = transforms.get_mut(entity) {
                                egui::CollapsingHeader::new("Transform")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for field in transform.data.inspect(ui) {
                                        if field.changed() {
                                            transform.data.update_matrix();
                                            transform.update_buffers(queue);
                                            if self.light_index.is_some() {
                                                light_manager.update_light_position(queue, self.light_index.unwrap(), transform.get_position())
                                            }
                                        }
                                    }
                                });
                            }
                            let mut materials_component = world.write_component::<MaterialComponent>();
                            if let Some(material_component) = materials_component.get_mut(entity) {
                                egui::CollapsingHeader::new("Material")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for field in material_component.inspect(ui) {
                                            if field.changed() {
                                                self.material = Some(materials[material_component.material_id].asset.borrow().clone());
                                                self.material_id = Some(material_component.material_id);
                                            }
                                        }
                                    
                                    if let Some(material) = &mut self.material {
                                        for field in material.inspect(ui) {
                                            if field.changed() { 
                                                materials[self.material_id.unwrap()].set_diffuse(material.diffuse);
                                            }
                                        }
                                    }
                                });
                            }
            
                            let mut meshes = world.write_component::<Mesh>();
                            if let Some(mesh) = meshes.get_mut(entity) {
                                egui::CollapsingHeader::new("Mesh")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        mesh.inspect(ui);
                                    });
                            }

                            let mut lights = world.write_component::<PointLight>();
                            if let Some(light) = lights.get_mut(entity) {
                                egui::CollapsingHeader::new("Point Light")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for field in light.inspect(ui) {
                                            if field.changed() {
                                                light_manager.update_light_data(queue, self.light_index.unwrap(), light.get_color())
                                            }
                                        }
                                    });
                            }
                        } else if let Some(material) = &mut self.material {
                            for field in material.inspect(ui) {
                                if field.changed() { 
                                    materials[self.material_id.unwrap()].set_diffuse(material.diffuse);
                                }
                            }
                        }
                    });
                });

            egui::SidePanel::right("World")
                .default_width(75.0)
                .resizable(true)
                .frame(frame)
                .show(&context, |ui| {
                    let panel = ui.vertical(|ui| {
                        ui.label("World");
                        ui.separator();
                        let names = world.read_component::<Name>();
                        let materials_component = world.read_component::<MaterialComponent>();

                        let point_light_component = world.read_component::<PointLight>();
                        let mut light_index = 0;

                        for entity in world.entities().join() {
                            let name = names.get(entity).unwrap();
                            let res = ui.add(egui::Button::new(name.0.to_string()));
                            if res.clicked() {
                                if let Some(material) = materials_component.get(entity) {
                                    self.material = Some(materials[material.material_id].asset.borrow().clone());
                                    self.material_id = Some(material.material_id);
                                } else {
                                    self.material = None;
                                    self.material_id = None;
                                }

                                if point_light_component.get(entity).is_some() {
                                    self.light_index = Some(light_index);
                                } else {
                                    self.light_index = None;
                                }
                                self.entity = Some(entity)
                            }
                            if point_light_component.get(entity).is_some() {
                                light_index += 1;
                            }
                        }
                    });

                    if panel.response.clicked() {
                        println!("hi");
                    }
                });
        })
    }

    pub fn draw(&mut self, context: &Context, world: &mut World, lights: &LightManager, materials: &mut Vec<Gpu<Material>>, window: &winit::window::Window, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        let egui_input = self.state.take_egui_input(window);
        let egui_output = self.world_inspect(egui_input, world, lights, materials, &context.queue);
        
        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
        });

        let clipped_primitives = self.context.tessellate(egui_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: egui_winit::native_pixels_per_point(window)
        };

        for (id, image) in egui_output.textures_delta.set {
            self.renderer.update_texture(&context.device, &context.queue, id, &image);
        }

        self.renderer.update_buffers(&context.device, &context.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

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

        context.queue.submit([encoder.finish()]);
        Ok(())
    }
}
