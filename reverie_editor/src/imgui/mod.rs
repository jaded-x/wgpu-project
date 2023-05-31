mod explorer;
mod viewport;

use std::{sync::Arc, path::PathBuf};

use reverie::{engine::{
    components::{
        transform::Transform, 
        name::Name,
        light::PointLight, material::MaterialComponent
    }, 
    light_manager::LightManager, registry::AssetType, model::Material, gpu::Gpu
}, util::cast_slice};
use specs::{*, WorldExt};

use reverie::engine::registry::Registry;

use imgui_inspector::ImguiInspect;
use crate::cursor::set_cursor;

use explorer::Explorer;

use self::viewport::Viewport;

pub struct Imgui {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,

    pub viewport: Viewport,
    pub explorer: Explorer,

    entity: Option<Entity>,
    light_index: Option<usize>,
}

impl Imgui {
    pub fn new(window: &winit::window::Window, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut context = imgui::Context::create();
        context.set_ini_filename(Some("imgui.ini".into()));
        context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window, imgui_winit_support::HiDpiMode::Rounded);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut context, device, queue, renderer_config);

        Self {
            context,
            platform,
            renderer,
            viewport: Viewport::new(device),
            explorer: Explorer::new(),

            entity: None,
            light_index: None,
        }
    }

    fn ui(&mut self, world: &specs::World, registry: &mut Registry, light_manager: &LightManager, queue: &wgpu::Queue, window: &winit::window::Window) {
        let ui = self.context.frame();

        ui.dockspace_over_main_viewport();
        
        ui.window("Performance").build(|| {
            ui.text(format!("{} FPS ({:.3}ms)", (ui.io().framerate as u32), (ui.io().delta_time * 1000.0)));
        });

        ui.window("Inspector")
            .build(|| {
                if let Some(material_path) = &self.explorer.selected_file {
                    let material_id = registry.get_id(material_path.to_path_buf());
                    match registry.metadata.get(&material_id).unwrap().asset_type  {
                        AssetType::Material => {
                            ui.text(material_path.file_name().unwrap().to_str().unwrap());
                            ui.separator();
                            let mut material_asset = self.explorer.material.as_ref().unwrap().asset.lock().unwrap();
                            let inspect = material_asset.imgui_inspect(ui);
                            if inspect[0] {
                                self.explorer.material.as_ref().unwrap().update_buffer(0, cast_slice(&[material_asset.diffuse]));
                                material_asset.save(material_path);
                            }

                            if inspect[1] || inspect[2] {
                                material_asset.save(material_path);
                                registry.reload_material(material_id);
                                for entity in world.entities().join() {
                                    let mut materials = world.write_component::<MaterialComponent>();
                                    if let Some(material) = materials.get_mut(entity) {
                                        if material.id == material_id {
                                            material.material = registry.get_material(material_id).unwrap();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(entity) = self.entity {
                    let names = world.read_component::<Name>();
                    ui.text(names.get(entity).unwrap().0.clone());
                    ui.separator();

                    let mut transforms = world.write_component::<Transform>();
                    if let Some(transform) = transforms.get_mut(entity) {
                        if ui.collapsing_header("Transform", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            if transform.data.imgui_inspect(ui).iter().any(|&value| value == true) {
                                transform.data.update_matrix();
                                transform.update_buffers(queue);
                                if self.light_index.is_some() {
                                    light_manager.update_light_position(queue, self.light_index.unwrap(), transform.get_position());
                                }
                            }
                        }
                    }

                    let mut lights = world.write_component::<PointLight>();
                    if let Some(light) = lights.get_mut(entity) {
                        if ui.collapsing_header("Point Light", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            if light.imgui_inspect(ui).iter().any(|&value| value == true) {
                                light_manager.update_light_data(queue, self.light_index.unwrap(), light.get_color());
                            }
                        }
                    }

                    let mut materials = world.write_component::<MaterialComponent>();
                    if let Some(material) = materials.get_mut(entity) {
                        if ui.collapsing_header("Material", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            let mut material_asset = material.material.asset.lock().unwrap();
                            if material_asset.imgui_inspect(ui).iter().any(|&value| value == true) {
                                material.material.update_diffuse_buffer(material_asset.diffuse);
                                
                            }
                        }
                    }
                }
            });

        ui.window("Objects").build(|| {
            let names = world.read_component::<Name>();
            let light_component = world.read_component::<PointLight>();

            let mut light_index = 0;

            for entity in world.entities().join() {
                let name = names.get(entity).unwrap();

                if ui.button(name.0.to_string()) {
                    self.entity = Some(entity);
                    self.explorer.selected_file = None;
                    self.explorer.material = None;

                    match light_component.get(entity) {
                        Some(_) => self.light_index = Some(light_index),
                        None => self.light_index = None,
                    }
                }

                if light_component.get(entity).is_some() {
                    light_index += 1;
                }
            }
        });

        self.viewport.ui(ui);
        self.explorer.ui(ui, registry);
        

        if ui.is_any_item_hovered() && !ui.is_any_item_active() {
            set_cursor(window, ui);
        }
    }

    pub fn draw(&mut self, world: &specs::World, registry: &mut Registry, light_manager: &LightManager, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, window: &winit::window::Window, encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError> {
        self.platform.prepare_frame(self.context.io_mut(), window).expect("Failed to prepare frame");

        self.ui(world, registry, light_manager, queue, window);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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

        self.renderer.render(self.context.render(), queue, device, &mut render_pass)
            .expect("rendering failed");

        drop(render_pass);

        Ok(())
    }

    pub async fn load_texture(&mut self, file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, id: usize) {
        let texture = reverie::engine::resources::load_texture(file_name, false, device, queue).await.unwrap().texture;
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let imgui_texture = imgui_wgpu::Texture::from_raw_parts(
            &device, 
            &self.renderer, 
            Arc::new(texture), 
            Arc::new(texture_view), 
            None, 
            Some(&imgui_wgpu::RawTextureConfig {
                label: Some("raw texture config"),
                sampler_desc: wgpu::SamplerDescriptor {
                    ..Default::default()
                }
            }), 
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.renderer.textures.replace(imgui::TextureId::new(id), imgui_texture);
    }
}