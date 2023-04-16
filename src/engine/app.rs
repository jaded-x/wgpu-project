use std::{rc::Rc, cell::RefCell};

use specs::prelude::*;

use crate::{util::cast_slice, engine::{model::Material, gpu::Gpu}};

use super::{
    camera::{Camera, CameraController, Projection},
    components::{
        mesh::Mesh,
        transform::Transform,
        material::MaterialComponent,
        renderable::Renderable,
        name::Name,
    }, 
    renderer::{Renderer, Pass},
    context::Context,
    egui::Egui,
    input::InputState,
    resources,
    texture,
    window::*, model::Model, light::Light,
};

pub struct App {
    pub context: Context,
    renderer: Renderer,
    input: InputState,
    egui: Egui,

    camera: Camera,
    camera_controller: CameraController,
    world: World,

    textures: Vec<Rc<texture::Texture>>,
    materials: Vec<Gpu<Material>>,
    models: Vec<Model>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        env_logger::init();

        let context = Context::new(&window.window).await;
        let renderer = Renderer::new(&context.device, &context.config); 

        let mut egui = Egui::new(&window.event_loop, &context);
        egui.state.set_pixels_per_point(window.window.scale_factor() as f32);

        let camera = Camera::new(&context.device, &renderer.camera_bind_group_layout, (0.0, 5.0, 10.0), cg::Deg(-90.0), cg::Deg(-20.0), 
            Projection::new(context.config.width, context.config.height, cg::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 0.2);

        let input = InputState::default();

        // 

        let default_diffuse_texture = Rc::new(texture::Texture::from_bytes(&context.device, &context.queue, include_bytes!("../../res/default_diffuse_texture.jpg"), "default_diffuse_texture.jpg").unwrap());
        let stone_tex = Rc::new(texture::Texture::from_bytes(&context.device, &context.queue, include_bytes!("../../res/cube-diffuse.jpg"), "cube-diffuse.jpg").unwrap());

        let textures = vec![default_diffuse_texture, stone_tex];

        let sphere_model = resources::load_model("sphere.obj", &context.device, &context.queue.clone(), &renderer.material_bind_group_layout).await.unwrap();
        let cube_model = resources::load_model("cube.obj", &context.device, &context.queue.clone(), &renderer.material_bind_group_layout).await.unwrap();
        let mut models = vec![cube_model, sphere_model];

        let green_material = Gpu::new(Rc::new(RefCell::new(Material::new(Some("Flat Color".to_string()), [0.0, 1.0, 0.0], textures[0].clone()))), context.device.clone(), renderer.material_bind_group_layout.clone(), context.queue.clone());
        let purple_stone = Gpu::new(Rc::new(RefCell::new(Material::new(Some("Stone".to_string()), [1.0, 0.0, 1.0], textures[1].clone()))), context.device.clone(), renderer.material_bind_group_layout.clone(), context.queue.clone());

        let mut materials = vec![green_material, purple_stone];

        materials[0].set_diffuse([0.6, 0.5, 0.4]);
        models[0].materials[0].set_diffuse([0.4, 0.4, 9.0]);

        let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Renderable>();
        world.register::<Name>();
        world.register::<Light>();
        world.create_entity()
            .with(Name::new("Cube"))
            .with(Transform::default())
            .with(Mesh::new(0))
            .with(MaterialComponent::new(0))
            .with(Renderable::new(&context.device, &renderer)).build();
        world.create_entity()
            .with(Name::new("Sphere"))
            .with(Transform::default())
            .with(Mesh::new(1))
            .with(MaterialComponent::new(0))
            .with(Renderable::new(&context.device, &renderer)).build();
        world.create_entity()
            .with(Name::new("Light"))
            .with(Transform::from_position(cg::vec3(0.0, 5.0, 0.0)))
            .with(Light::new([1.0, 1.0, 1.0]))
            .build();

        { // update buffers
            let mut renderables = world.write_component::<Renderable>();
            let transforms = world.read_component::<Transform>();
            for (transform, renderable) in (&transforms, &mut renderables).join() {
                renderable.update_transform_buffer(&context.queue, transform);
            }
        }

        Self {
            context,
            input,
            camera,
            camera_controller,
            world,
            renderer,
            egui,
            textures,
            materials,
            models,
        }
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.context.config.width = new_window_size.width;
            self.context.config.height = new_window_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.renderer.resize(&self.context.device, &self.context.config);
            self.camera.projection.resize(new_window_size.width, new_window_size.height);
        }
    }

    fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt, &self.input);
        self.camera.update_uniform();
        self.context.queue.write_buffer(&self.camera.buffer, 0, cast_slice(&[self.camera.uniform]));

        // update buffers
        let mut renderables = self.world.write_component::<Renderable>();
        let transforms = self.world.read_component::<Transform>();

        for (transform, renderable) in (&transforms, &mut renderables).join() {
            renderable.update_transform_buffer(&self.context.queue, transform);
        }
    }

    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.draw(&self.context, &view, &mut self.world, window, &self.camera, &self.models, &mut self.materials)?;
        self.egui.draw(&self.context, &mut self.world, &mut self.materials, window, &view)?;

        output.present();
        Ok(())
    }
}

use winit::{
    event::*,
    event_loop::ControlFlow,
};

pub async fn run() {
    let window = Window::new();
    let mut app = App::new(&window).await;

    let mut last_render_time = instant::Instant::now();
    
    window.event_loop.run(move |event, _, control_flow| match event {
        event if app.input.update(&event) => {}
        Event::WindowEvent { event, .. } => match event {
            e if app.egui.state.on_event(&app.egui.context, &e).consumed => {}
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
    //         app.render(window.unwrap());
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