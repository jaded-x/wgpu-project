use std::sync::Arc;

use reverie::engine::registry::Registry;
use reverie::engine::scene::Scene;
use reverie::engine::texture::Texture;
use reverie::util::{cast_slice, res};

use reverie::engine::{
    camera::{Camera, CameraController, Projection},
    renderer::{Renderer, Pass},
    context::Context,
    input::InputState,
    window::*, 
};

pub struct App {
    pub context: Context,
    renderer: Renderer,
    input: InputState,
    imgui: Imgui,

    camera: Camera,
    camera_controller: CameraController,

    scene: Scene,

    //watcher: FileWatcher,

    registry: Registry,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        env_logger::init();

        let context = Context::new(&window.window).await;
        let mut imgui = Imgui::new(&window.window, &context.device, &context.queue);

        let renderer = Renderer::new(&context.device, &context.config, &imgui.viewport.texture.size()); 

        let mut registry = Registry::new(context.device.clone(), context.queue.clone(), imgui.renderer.clone());

        Texture::load_defaults(&context.device, &context.queue);

        let input = InputState::default();

        // let mut watcher = FileWatcher::new().unwrap();
        // watcher.watch(Path::new("res")).unwrap();

        imgui.load_texture("src/imgui/textures/folder.png", &context.device, &context.queue, 64, 64, 3);
        imgui.load_texture("src/imgui/textures/file.png", &context.device, &context.queue, 64, 64, 4);
        imgui.load_texture("src/imgui/textures/background.png", &context.device, &context.queue, 1, 1, 5);

        let scene = Scene::new(res("scenes/first.revscene"), &mut registry, &context.device);

        let camera = Camera::new(&context.device, &renderer.camera_bind_group_layout, (0.0, 5.0, 10.0), cg::Deg(-90.0), cg::Deg(-20.0), 
            Projection::new(context.config.width, context.config.height, cg::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 0.5);

        Self {
            context,
            input,
            camera,
            camera_controller,
            scene,
            renderer,
            imgui,
            //watcher,
            registry
        }
    }

    fn resize_viewport(&mut self) {
        let extent = wgpu::Extent3d {
            width: self.imgui.viewport.size[0],
            height: self.imgui.viewport.size[1],
            depth_or_array_layers: 1,
        };
        
        self.imgui.viewport.texture  = Arc::new(self.context.device.create_texture(&wgpu::TextureDescriptor {
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
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.context.config.width = new_window_size.width;
            self.context.config.height = new_window_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
        }
    }

    fn update(&mut self, dt: instant::Duration) {
        if self.imgui.viewport.active {
            self.camera_controller.update_camera(&mut self.camera, dt, &self.input);
        }
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

        if &[self.imgui.viewport.texture.width(), self.imgui.viewport.texture.height()] != &self.imgui.viewport.size {
            self.resize_viewport();
        }

        let viewport_view = self.imgui.viewport.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.draw(&viewport_view, &mut self.scene, &self.camera, &mut encoder)?;
        
        let texture = imgui_wgpu::Texture::from_raw_parts(
            &self.context.device, 
            &self.imgui.renderer.lock().unwrap(), 
            self.imgui.viewport.texture.clone(), 
            Arc::new(viewport_view), 
            None, 
            Some(&imgui_wgpu::RawTextureConfig {
                label: Some("raw texture config"),
                sampler_desc: wgpu::SamplerDescriptor {
                    ..Default::default()
                }
            }), 
            wgpu::Extent3d {
                width: self.imgui.viewport.texture.width(),
                height: self.imgui.viewport.texture.width(),
                depth_or_array_layers: 1,
            },
        );
        
        self.imgui.renderer.lock().unwrap().textures.replace(imgui::TextureId::new(2), texture);
        self.imgui.draw(&mut self.scene, &mut self.registry, &self.context.device, &self.context.queue, &view, &window, &mut encoder)?;

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