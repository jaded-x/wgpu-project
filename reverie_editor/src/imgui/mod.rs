mod explorer;
mod viewport;
mod hierarchy;

use std::{sync::{Arc, Mutex}, path::PathBuf};

use reverie::engine::{
    components::{
        transform::Transform, 
        name::Name,
        light::{PointLight, DirectionalLight}, material::MaterialComponent, mesh::Mesh, ComponentDefault, TypeName
    }, registry::AssetType, texture::Texture, scene::Scene,
};
use specs::{*, WorldExt};

use reverie::engine::registry::Registry;

use imgui_inspector::ImguiInspect;
use crate::cursor::set_cursor;

use explorer::Explorer;

use self::{viewport::Viewport, hierarchy::Hierarchy};

pub struct Imgui {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub renderer: Arc<Mutex<imgui_wgpu::Renderer>>,

    pub viewport: Viewport,
    pub explorer: Explorer,
    pub hierarchy: Hierarchy,

    directional_light_index: Option<usize>,
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

        let renderer = Arc::new(Mutex::new(imgui_wgpu::Renderer::new(&mut context, device, queue, renderer_config)));

        Self {
            context,
            platform,
            renderer,
            viewport: Viewport::new(device),
            explorer: Explorer::new(),
            hierarchy: Hierarchy::new(),

            directional_light_index: None,
        }
    }

    fn ui(&mut self, device: &wgpu::Device, scene: &mut Scene, registry: &mut Registry, queue: &wgpu::Queue, window: &winit::window::Window) {
        let ui = self.context.frame();

        ui.dockspace_over_main_viewport();
        
        ui.window("Performance").build(|| {
            ui.text(format!("{} FPS ({:.3}ms)", (ui.io().framerate as u32), (ui.io().delta_time * 1000.0)));
        });

        ui.window("Inspector")
            .build(|| {
                if let Some(material_path) = &self.explorer.selected_file.clone() {
                    let material_id = registry.get_id(material_path.to_path_buf());
                    match registry.metadata.get(&material_id).unwrap().asset_type  {
                        AssetType::Material => {
                            ui.text(material_path.file_name().unwrap().to_str().unwrap());
                            ui.separator();
                            let mut material_asset = self.explorer.material.as_ref().unwrap().asset.lock().unwrap();
                            if material_asset.imgui_inspect(ui).iter().any(|&value| value == true) {
                                material_asset.save(material_path);
                                drop(material_asset);
                                registry.load_material(material_id, true);
                                update_entity_material(&scene.world, material_id, registry);
                                self.explorer.material = registry.get_material(material_id);
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(entity) = self.hierarchy.entity {
                    let mut names = scene.world.write_component::<Name>();
                    ui.input_text("##entity name", &mut names.get_mut(entity).unwrap().0).build();
                    ui.separator();
                    {
                        let mut transforms = scene.world.write_component::<Transform>();

                        let mut parent_id = None;
                        let mut parent_matrix: Option<cg::Matrix4<f32>> = None;
                        if let Some(transform) = transforms.get_mut(entity) {
                            if let Some(id) = transform.data.parent {
                                parent_id = Some(id);
                            }
                        }
                        if let Some(id) = parent_id {
                            if let Some(transform) = transforms.get(scene.world.entities().entity(id as u32)) {
                                parent_matrix = Some(transform.get_matrix());
                            }
                        }

                        if let Some(transform) = transforms.get_mut(entity) {
                            if ui.collapsing_header("Transform", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                                if transform.data.imgui_inspect(ui).iter().any(|&value| value == true) {
                                    transform.data.update_matrix(parent_matrix);
                                    transform.update_buffers(queue);
                                    if self.hierarchy.point_light_index.is_some() {
                                        scene.light_manager.update_light_position(queue, self.hierarchy.point_light_index.unwrap(), transform.get_position());
                                    }
                                    let matrix = transform.get_matrix().clone();
                                    let children = transform.data.children.clone();
                                    drop(transform);
                                    for child in children {
                                        let child_transform = transforms.get_mut(scene.world.entities().entity(child)).unwrap();
                                        if child_transform.data.parent.unwrap() == entity.id() {
                                            child_transform.data.update_matrix(Some(matrix));
                                            child_transform.update_buffers(queue);
                                        }
                                    }
                                }
                            }
                        }

                        let mut lights = scene.world.write_component::<PointLight>();
                        if let Some(light) = lights.get_mut(entity) {
                            if ui.collapsing_header("Point Light", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                                if light.imgui_inspect(ui).iter().any(|&value| value == true) {
                                    scene.light_manager.update_light_color(queue, self.hierarchy.point_light_index.unwrap(), light.get_color());
                                }
                            }
                        }

                        let mut directional_lights = scene.world.write_component::<DirectionalLight>();
                        if let Some(light) = directional_lights.get_mut(entity) {
                            if ui.collapsing_header("Directional Light", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                                if light.imgui_inspect(ui).iter().any(|&value| value == true) {
                                    scene.light_manager.update_directional_data(queue, self.directional_light_index.unwrap(), light.direction, light.color)
                                }
                            }
                        }

                        let mut materials = scene.world.write_component::<MaterialComponent>();
                        if let Some(material) = materials.get_mut(entity) {
                            let material_id = material.id.clone();
                            if ui.collapsing_header("Material", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                                ui.button("material");
                                match ui.drag_drop_target() {
                                    Some(target) => {
                                        match target.accept_payload::<Option<usize>, _>(AssetType::Material.to_string(), imgui::DragDropFlags::empty()) {
                                            Some(Ok(payload_data)) => {
                                                material.id = payload_data.data.unwrap();
                                                material.material = registry.get_material(material.id).unwrap();
                                            },
                                            Some(Err(e)) => {
                                                println!("{}", e);
                                            },
                                            _ => {},
                                        }
                                    },
                                    _ => {},
                                }

                                let material_path = registry.get_filepath(material.id);
                                ui.text(material_path.file_name().unwrap().to_str().unwrap());
                                ui.separator();
                                let mut material_asset = material.material.asset.lock().unwrap();
                                if material_asset.floats.imgui_inspect(ui).iter().any(|&value| value == true) {
                                    material.material.update_floats(material_asset.floats);
                                }
                                if material_asset.imgui_inspect(ui).iter().any(|&value| value == true) {
                                    material_asset.save(&material_path);
                                    drop(material_asset);
                                    registry.load_material(material_id, true);
                                    drop(materials);
                                    update_entity_material(&scene.world, material_id, registry);
                                    self.explorer.material = registry.get_material(material_id);
                                }
                            }
                        }

                        let mut meshes = scene.world.write_component::<Mesh>();
                        if let Some(mesh) = meshes.get_mut(entity) {
                            if ui.collapsing_header("Mesh", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                                ui.button("mesh");
                                match ui.drag_drop_target() {
                                    Some(target) => {
                                        match target.accept_payload::<Option<usize>, _>(AssetType::Mesh.to_string(), imgui::DragDropFlags::empty()) {
                                            Some(Ok(payload_data)) => {
                                                mesh.id = payload_data.data.unwrap();
                                                mesh.mesh = registry.get_mesh(mesh.id).unwrap();
                                            },
                                            Some(Err(e)) => {
                                                println!("{}", e);
                                            },
                                            _ => {},
                                        }
                                    }
                                    _ => {},
                                }
                            }
                        }
                    }
                    drop(names);

                    ui.popup("components", || {
                        ui.text("Add Component");
                        add_component::<Name>(ui, scene, entity, device, registry, None);
                        add_component::<Transform>(ui, scene, entity, device, registry, None);
                        add_component::<MaterialComponent>(ui, scene, entity, device, registry, None);
                        add_component::<Mesh>(ui, scene, entity, device, registry, None);
                        add_component::<PointLight>(ui, scene, entity, device, registry, self.hierarchy.point_light_index);
                        add_component::<DirectionalLight>(ui, scene, entity, device, registry, self.directional_light_index);
                    });

                    if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                        ui.open_popup("components")
                    }
                }
            });

        self.hierarchy.ui(ui, scene, &mut self.explorer);

        self.viewport.ui(ui, scene, registry, device);
        self.explorer.ui(ui, registry);
        
        if self.explorer.selected_file.is_some() {
            self.hierarchy.entity = None;
        }

        set_cursor(window, ui);
    }

    pub fn draw(&mut self, scene: &mut Scene, registry: &mut Registry, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, window: &winit::window::Window, encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError> {
        self.platform.prepare_frame(self.context.io_mut(), window).expect("Failed to prepare frame");

        self.ui(device, scene, registry, queue, window);

        let mut renderer_lock = self.renderer.lock().unwrap();
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

        renderer_lock.render(self.context.render(), queue, device, &mut render_pass)
            .expect("rendering failed");
        drop(render_pass);

        Ok(())
    }

    pub fn load_texture(&mut self, file_name: &PathBuf, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, id: usize) {
        let bytes = std::fs::read(file_name).unwrap();
        let texture = Texture::from_bytes(device, queue, &bytes, file_name.to_str().unwrap(), false).unwrap();
        let imgui_texture = imgui_wgpu::Texture::from_raw_parts(
            &device, 
            &self.renderer.lock().unwrap(), 
            Arc::new(texture.texture), 
            Arc::new(texture.view), 
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
        self.renderer.lock().unwrap().textures.replace(imgui::TextureId::new(id), imgui_texture);
    }
}

fn update_entity_material(world: &World, id: usize, registry: &mut Registry) {
    for entity in world.entities().join() {
        let mut materials = world.write_component::<MaterialComponent>();
        if let Some(material) = materials.get_mut(entity) {
            if material.id == id {
                material.material = registry.get_material(id).unwrap();
            }
        }
    }
}

use std::string::ToString;

fn add_component<'a, T: ComponentDefault + specs::Component>(ui: &'a imgui::Ui, scene: &mut Scene, entity: Entity, device: &wgpu::Device, registry: &mut Registry, index: Option<usize>) where T: TypeName {
    if ui.button(T::type_name()) {
        let mut components = scene.world.write_storage::<T>();
        if let Some(_) = components.get(entity) {
            components.remove(entity);
            if T::type_name() == "point_light" {
                scene.light_manager.remove_point_light(device, index.unwrap())
            } else if T::type_name() == "directional_light"  {
                scene.light_manager.remove_directional_light(device, index.unwrap())
            }
        } else {
            components.insert(entity, T::default(device, registry)).expect(&format!("Failed to add component: {}", T::type_name()));
            drop(components);
            if T::type_name() == "point_light" {
                let transforms = scene.world.read_component::<Transform>();
                let lights = scene.world.read_component::<PointLight>();
                scene.light_manager.add_point_light(device, transforms.get(entity).unwrap(), lights.get(entity).unwrap());
            } else if T::type_name() == "directional_light" {
                let lights = scene.world.read_component::<DirectionalLight>();
                scene.light_manager.add_directional_light(device, lights.get(entity).unwrap());
            }
        }
    }
}