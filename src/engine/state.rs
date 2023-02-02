use wgpu::util::DeviceExt;
use winit::{
    event::*,
};

use crate::{util::cast_slice, engine::{components::renderable::Renderable, context::Context}};

use super::{
    texture::Texture,
    camera::{Camera, CameraController, Projection},
    components::{*, mesh::{Vert, Mesh}}, 
    window::{Window, WindowEvents}, 
    renderer::{Renderer, Pass},
};

use specs::prelude::*;

use transform::Transform;

pub struct State {
    context: Context,
    renderer: Renderer,
    
    camera: Camera,
    camera_controller: CameraController,
    world: specs::World,
}

impl State {
    async fn new(window: &Window) -> Self {
        let context = Context::new(window).await;
        let renderer = Renderer::new(&context.device, &context.queue, &context.config);

        let camera_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera = Camera::new(&context.device, &camera_bind_group_layout, (0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0), 
            Projection::new(context.config.width, context.config.height, cgmath::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 1.0);

        // let sphere_model = resources::load_model("sphere.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let cube_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let models = vec![sphere_model, cube_model];

        const VERTICES: &[Vert] = &[
            Vert { position: [0.0, 0.5, 0.0]},
            Vert { position: [-0.5, -0.5, 0.0]},
            Vert { position: [0.5, -0.5, 0.0]},
        ];

        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        const INDICES: &[u16] = &[
            0, 1, 2
        ];

        let index_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let index_count = INDICES.len() as u32;

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Mesh>();
        world.register::<Renderable>();
        world.create_entity()
            .with(Transform::default())
            .with(Mesh::new(vertex_buffer, index_buffer, index_count))
            .with(Renderable::new(&context.device)).build();

        Self {
            context,
            camera,
            camera_controller,
            world,
            renderer,
        }
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.context.window_size = new_window_size;
            self.context.config.width = new_window_size.width;
            self.context.config.height = new_window_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.camera.projection.resize(new_window_size.width, new_window_size.height);
            self.renderer.depth_texture = Texture::create_depth_texture(&self.context.device, &self.context.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera, dt);
        // self.camera.update_uniform();
        // self.queue.write_buffer(&self.camera.buffer, 0, cast_slice(&[self.camera.uniform]));
        
        let mut transforms = self.world.write_component::<Transform>();
        for transform in (&mut transforms).join() {
            transform.position.x = transform.position.x + 0.01;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.draw(&self.context.surface, &self.context.device, &self.context.queue, &self.world)
    }

}

pub async fn run() {
    env_logger::init();
    let window = Window::new();

    let mut state = State::new(&window).await;

    window.run(move |event| match event {
        WindowEvents::Resized { width, height } => {
            state.resize(winit::dpi::PhysicalSize { width, height });
        }
        WindowEvents::Draw => {
            state.update();
            state.render();
        }
        WindowEvents::Keyboard {
            state,
            virtual_keycode,
        } => {}
    });
}