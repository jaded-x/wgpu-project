use std::sync::Arc;

use specs::prelude::*;
use wgpu::util::DeviceExt;
use winit::event::*;

use crate::util::cast_slice;

use super::{
    camera::{Camera, CameraController, Projection},
    components::{
        mesh::{Vert, Mesh},
        transform::Transform,
        material::MaterialComponent,
        renderable::Renderable,
        name::Name,
    }, 
    renderer::{Renderer, Pass},
    context::Context,
    egui::Egui,
    input::InputState,
    texture,
    material::Material,
};

pub struct State {
    pub context: Context,
    renderer: Renderer,
    window: winit::window::Window,

    camera: Camera,
    camera_controller: CameraController,
    world: World,
}

impl State {
    async fn new(window: winit::window::Window) -> Self {
        let context = Context::new(&window).await;
        let renderer = Renderer::new(&context.device, &context.config); 

        let camera = Camera::new(&context.device, &renderer.camera_bind_group_layout, (0.0, 5.0, 10.0), cg::Deg(-90.0), cg::Deg(-20.0), 
            Projection::new(context.config.width, context.config.height, cg::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 0.2);

        // let sphere_model = resources::load_model("sphere.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let cube_model = resources::load_model("cube.obj", &context.device, &context.queue, &texture_bind_group_layout).await.unwrap();
        // let models = vec![sphere_model, cube_model];

        const VERTICES: &[Vert] = &[
            Vert { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0]},
            Vert { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 0.0]},
            Vert { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 1.0]},
            Vert { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 1.0]},
        ];
        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let vertex_buffer2 = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        const INDICES: &[u16] = &[
            0, 1, 2,
            2, 3, 0
        ];
        let index_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let index_buffer2 = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let index_count = INDICES.len() as u32;

        let texture = texture::Texture::from_bytes(&context.device, &context.queue, include_bytes!("../../res/cube-diffuse.jpg"), "cube_diffuse.jpg").unwrap();
        let mut material = Material::new(&context.device, &renderer);
        material.set_texture(texture, &context.device, &renderer);
        let mat = Arc::new(material);

        let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Renderable>();
        world.register::<Name>();
        world.create_entity()
            .with(Name::new("Square 1"))
            .with(Transform::default())
            .with(Mesh::new(vertex_buffer, index_buffer, index_count))
            .with(MaterialComponent::new(Arc::clone(&mat)))
            .with(Renderable::new(&context.device, &renderer)).build();
        world.create_entity()
            .with(Name::new("Square 2"))
            .with(Transform::default())
            .with(Mesh::new(vertex_buffer2, index_buffer2, index_count))
            .with(MaterialComponent::new(Arc::clone(&mat)))
            .with(Renderable::new(&context.device, &renderer)).build();

        { // update buffer
            let mut renderables = world.write_component::<Renderable>();
            let transforms = world.read_component::<Transform>();
            let materials = world.read_component::<MaterialComponent>();
            for (transform, material, renderable) in (&transforms, &materials, &mut renderables).join() {
                renderable.update_transform_buffer(&context.queue, transform);
            }
        }

        Self {
            context,
            window,
            camera,
            camera_controller,
            world,
            renderer,
        }
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.context.config.width = new_window_size.width;
            self.context.config.height = new_window_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.camera.projection.resize(new_window_size.width, new_window_size.height);
        }
    }

    fn update(&mut self, dt: instant::Duration, input: &InputState) {
        self.camera_controller.update_camera(&mut self.camera, dt, input);
        self.camera.update_uniform();
        self.context.queue.write_buffer(&self.camera.buffer, 0, cast_slice(&[self.camera.uniform]));

        // update buffers
        let mut renderables = self.world.write_component::<Renderable>();
        let transforms = self.world.read_component::<Transform>();
        let materials = self.world.read_component::<MaterialComponent>();
        for (transform, material, renderable) in (&transforms, &materials, &mut renderables).join() {
            renderable.update_transform_buffer(&self.context.queue, transform);
        }
    }

    fn render(&mut self, egui: Option<&mut Egui>) -> Result<(), wgpu::SurfaceError> {
        self.renderer.draw(&self.context, &mut self.world, &self.window, egui, &self.camera)
    }


}

use winit::{
    event_loop::{ControlFlow, EventLoop},
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .with_title("wgpu")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut input = InputState::default();
    let mut state = State::new(window).await;

    let mut egui = Egui::new(&event_loop, &state.context);
    egui.state.set_pixels_per_point(state.window.scale_factor() as f32);

    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        event if input.update(&event) => {}
        Event::WindowEvent { event, .. } => match event {
            e if egui.state.on_event(&egui.context, &e).consumed => {}
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(*new_inner_size);
            }
            _ => {}
        }
        Event::RedrawRequested(_) => {
            let now = instant::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;
            state.update(dt, &input);

            input.finish_frame();
            match state.render(Some(&mut egui)) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.window.inner_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{e:?}"),
            };
        }
        Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        _ => {}
    });
}