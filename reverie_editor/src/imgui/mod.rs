mod explorer;

use std::sync::Arc;

use reverie::engine::{
    components::{
        transform::Transform, 
        name::Name,
        light::PointLight, material::MaterialComponent
    }, 
    light_manager::LightManager
};
use specs::{*, WorldExt};

use imgui_inspector::{ImguiInspect, InspectTexture};
use crate::cursor::set_cursor;

use explorer::Explorer;

pub struct Imgui {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,

    pub viewport_texture: Arc<wgpu::Texture>,
    pub viewport_size: [u32; 2],
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

        let viewport_texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: 100,
                height: 100,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }));

        let explorer = Explorer::new();

        Self {
            context,
            platform,
            renderer,
            viewport_size: [viewport_texture.width(), viewport_texture.height()],
            viewport_texture,
            explorer,

            entity: None,
            light_index: None,
        }
    }

    fn ui(&mut self, world: &specs::World, light_manager: &LightManager, queue: &wgpu::Queue, window: &winit::window::Window) {
        let ui = self.context.frame();

        ui.dockspace_over_main_viewport();
        
        ui.window("Performance").build(|| {
            ui.text(format!("{} FPS ({:.3}ms)", (ui.io().framerate as u32), (ui.io().delta_time * 1000.0)));
        });

        ui.window("Inspector")
            .build(|| {
                if let Some(entity) = self.entity {
                    let names = world.read_component::<Name>();
                    ui.text(names.get(entity).unwrap().0.clone());
                    ui.separator();

                    let mut transforms = world.write_component::<Transform>();
                    if let Some(transform) = transforms.get_mut(entity) {
                        if ui.collapsing_header("Transform", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            if transform.data.imgui_inspect(ui) {
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
                            if light.imgui_inspect(ui) {
                                light_manager.update_light_data(queue, self.light_index.unwrap(), light.get_color());
                            }
                        }
                    }

                    let mut materials = world.write_component::<MaterialComponent>();
                    if let Some(material) = materials.get_mut(entity) {
                        if ui.collapsing_header("Material", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            // let mut material_asset = material.material.asset.lock().unwrap();
                            // if material_asset.imgui_inspect(ui) {
                            //     material.material.update_diffuse_buffer(material_asset.diffuse);
                            // }
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

        ui.window("Viewport").build(|| {
            self.viewport_size = [ui.content_region_avail()[0] as u32, ui.content_region_avail()[1] as u32];
            imgui::Image::new(imgui::TextureId::new(2), ui.content_region_avail()).build(ui);
        });

        self.explorer.ui(ui);
        

        if ui.is_any_item_hovered() && !ui.is_any_item_active() {
            set_cursor(window, ui);
        }
    }

    pub fn draw(&mut self, world: &specs::World, light_manager: &LightManager, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, window: &winit::window::Window, encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError> {
        self.platform.prepare_frame(self.context.io_mut(), window).expect("Failed to prepare frame");

        self.ui(world, light_manager, queue, window);

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