use std::sync::Arc;

use reverie::engine::registry::Registry;
use reverie::engine::texture::Texture;
use specs::prelude::*;

use reverie::util::{cast_slice, res};

use reverie::engine::{
    camera::{Camera, CameraController, Projection},
    components::{
        mesh::Mesh,
        transform::{Transform, TransformData},
        material::MaterialComponent,
        name::Name,
        light::PointLight,
    }, 
    renderer::{Renderer, Pass},
    context::Context,
    input::InputState,
    resources,
    window::*, 
    light_manager::LightManager,
};

pub struct App {
    pub context: Context,
    renderer: Renderer,
    input: InputState,
    imgui: Imgui,

    camera: Camera,
    camera_controller: CameraController,
    world: World,

    light_manager: LightManager,
    //watcher: FileWatcher,

    registry: Registry,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        env_logger::init();

        let context = Context::new(&window.window).await;
        let mut imgui = Imgui::new(&window.window, &context.device, &context.queue);

        let renderer = Renderer::new(&context.device, &context.config, &imgui.viewport_texture.size()); 

        let mut registry = Registry::new(context.device.clone(), context.queue.clone());

        Texture::load_defaults(&context.device, &context.queue);

        let camera = Camera::new(&context.device, &renderer.camera_bind_group_layout, (0.0, 5.0, 10.0), cg::Deg(-90.0), cg::Deg(-20.0), 
            Projection::new(context.config.width, context.config.height, cg::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 0.5);

        let input = InputState::default();

        // let mut watcher = FileWatcher::new().unwrap();
        // watcher.watch(Path::new("res")).unwrap();

        let plane_model = resources::load_mesh("meshes/plane.obj", &context.device, &context.queue.clone(), &Renderer::get_material_layout()).await.unwrap();
        let cube_model = resources::load_mesh("meshes/cube.obj", &context.device, &context.queue.clone(), &Renderer::get_material_layout()).await.unwrap();

        let basic_material_id = registry.get_id(res("materials/basic.revmat"));

        let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Name>();
        world.register::<PointLight>();
        world.create_entity()
            .with(Name::new("Plane"))
            .with(Transform::new(TransformData::new(cg::vec3(0.0, 0.0, 0.0), cg::vec3(90.0, 0.0, 0.0), cg::vec3(1.0, 1.0, 1.0)), &context.device, &renderer.transform_bind_group_layout))
            .with(Mesh::new(plane_model[0].clone()))
            .with(MaterialComponent::new(basic_material_id, &mut registry))
            .build();
        world.create_entity()
            .with(Name::new("Cube"))
            .with(Transform::new(TransformData::new(cg::vec3(-2.0, 0.0, -2.0), cg::vec3(0.0, 0.0, 0.0), cg::vec3(1.0, 1.0, 1.0)), &context.device, &renderer.transform_bind_group_layout))
            .with(Mesh::new(cube_model[0].clone()))
            .with(MaterialComponent::new(basic_material_id, &mut registry))
            .build();
        world.create_entity()
            .with(Name::new("Light 1"))
            .with(Transform::new(TransformData::new(cg::vec3(0.0, 0.0, 1.0), cg::vec3(0.0, 0.0, 0.0), cg::vec3(0.1, 0.1, 0.1)), &context.device, &renderer.transform_bind_group_layout))
            .with(PointLight::new([1.0, 1.0, 1.0]))
            .with(Mesh::new(cube_model[0].clone()))
            .with(MaterialComponent::new(basic_material_id, &mut registry))
            .build();
        world.create_entity()
            .with(Name::new("Light 2"))
            .with(Transform::new(TransformData::new(cg::vec3(0.0, 0.0, 1.0), cg::vec3(0.0, 0.0, 0.0), cg::vec3(0.1, 0.1, 0.1)), &context.device, &renderer.transform_bind_group_layout))
            .with(PointLight::new([1.0, 1.0, 1.0]))
            .with(Mesh::new(cube_model[0].clone()))
            .with(MaterialComponent::new(18107631171250797795, &mut registry))
            .build();

        let light_manager = LightManager::new(&context.device, &renderer.light_bind_group_layout, &world);

        imgui.load_texture("../src/imgui/textures/folder.png", &context.device, &context.queue, 64, 64, 3).await;
        imgui.load_texture("../src/imgui/textures/file.png", &context.device, &context.queue, 64, 64, 4).await;

        Self {
            context,
            input,
            camera,
            camera_controller,
            world,
            renderer,
            imgui,
            light_manager,
            //watcher,
            registry
        }
    }

    fn resize_imgui(&mut self) {
        let extent = wgpu::Extent3d {
            width: self.imgui.viewport_size[0],
            height: self.imgui.viewport_size[1],
            depth_or_array_layers: 1,
        };
        
        self.imgui.viewport_texture  = Arc::new(self.context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }));
        self.renderer.resize(&self.context.device, &extent);
        self.camera.projection.resize(extent.width, extent.height);
        self.camera.projection.aspect;
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.context.config.width = new_window_size.width;
            self.context.config.height = new_window_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
        }
    }

    fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt, &self.input);
        self.camera.update_uniform();
        self.context.queue.write_buffer(&self.camera.buffer, 0, cast_slice(&[self.camera.uniform]));
    
        //self.watcher.handle_events(&mut self.textures);
    }

    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
        });

        if &[self.imgui.viewport_texture.width(), self.imgui.viewport_texture.height()] != &self.imgui.viewport_size {
            self.resize_imgui();
        }

        let viewport_view = self.imgui.viewport_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.draw(&viewport_view, &mut self.world, &self.camera, &self.light_manager, &mut encoder)?;
        
        let texture = imgui_wgpu::Texture::from_raw_parts(
            &self.context.device, 
            &self.imgui.renderer, 
            self.imgui.viewport_texture.clone(), 
            Arc::new(viewport_view), 
            None, 
            Some(&imgui_wgpu::RawTextureConfig {
                label: Some("raw texture config"),
                sampler_desc: wgpu::SamplerDescriptor {
                    ..Default::default()
                }
            }), 
            wgpu::Extent3d {
                width: self.imgui.viewport_texture.width(),
                height: self.imgui.viewport_texture.width(),
                depth_or_array_layers: 1,
            },
        );
        
        self.imgui.renderer.textures.replace(imgui::TextureId::new(2), texture);
        self.imgui.draw(&self.world, &mut self.registry, &self.light_manager, &self.context.device, &self.context.queue, &view, &window, &mut encoder)?;

        self.context.queue.submit([encoder.finish()]);
        output.present();
        Ok(())
    }
}

use winit::{
    event::*,
    event_loop::ControlFlow,
};

use crate::imgui::Imgui;

pub async fn run() {
    let window = Window::new();
    let mut app = App::new(&window).await;
    let mut last_render_time = instant::Instant::now();
    
    window.event_loop.run(move |event, _, control_flow| {
        app.imgui.platform.handle_event(app.imgui.context.io_mut(), &window.window, &event);
        
        match event {
            event if app.input.update(&event) => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    app.resize(size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    app.resize(*new_inner_size);
                }
                _ => {}
            }
            Event::RedrawRequested(_) => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                app.imgui.context.io_mut().delta_time = dt.as_secs_f32();

                app.update(dt);

                app.input.finish_frame();
                match app.render(&window.window) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.resize(window.window.inner_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{e:?}"),
                };
            }
            Event::MainEventsCleared => {
                window.window.request_redraw();
            }
            _ => {}
        }
    });

    // window.run(move |event, window| match event {
    //     Events::Resized { width, height } => {
    //         app.resize(winit::dpi::PhysicalSize { width, height });
    //     }
    //     Events::Draw => {
    //         let now = instant::Instant::now();
    //         let dt = now - last_render_time;
    //         last_render_time = now;

    //         app.update(dt);

    //         app.input.finish_frame();
    //         match app.render(window.unwrap()) {
    //             Ok(_) => {}
    //             Err(wgpu::SurfaceError::Lost) => app.resize(window.unwrap().inner_size()),
    //             Err(e) => eprintln!("{e:?}"),
    //         }
    //     }
    //     Events::KeyboardInput { state, virtual_keycode } => {
    //         app.input.update_keyboard(state, virtual_keycode);
    //     }
    //     Events::MouseInput { state, button } => {
    //         app.input.update_mouse_input(state, button);
    //     }
    //     Events::MouseMotion { delta } => {
    //         app.input.update_mouse_motion(delta);
    //     }
    //     Events::MouseWheel { delta } => {
    //         app.input.update_mouse_wheel(delta);
    //     }
    // });

}