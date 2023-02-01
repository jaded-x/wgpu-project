use wgpu::util::DeviceExt;
use winit::{
    event::*,
};

use crate::{util::cast_slice, engine::components::renderable::Renderable};

use super::{
    texture::Texture,
    camera::{Camera, CameraController, Projection},
    instance::Instance, 
    components::{*, mesh::{Vert, Mesh}}, 
    window::{Window, WindowEvents}, 
    renderer::{Renderer, Pass},
};

use specs::prelude::*;

use transform::Transform;

pub struct State {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_controller: CameraController,
    world: specs::World,

    renderer: Renderer,
}

impl State {
    async fn new(window: &Window) -> Self {
        let window_size = window.window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window.window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features:wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            },
            None,
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera = Camera::new(&device, &camera_bind_group_layout, (0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0), 
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0));
        let camera_controller = CameraController::new(4.0, 1.0);

        // let sphere_model = resources::load_model("sphere.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let cube_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let models = vec![sphere_model, cube_model];

        const VERTICES: &[Vert] = &[
            Vert { position: [0.0, 0.5, 0.0]},
            Vert { position: [-0.5, -0.5, 0.0]},
            Vert { position: [0.5, -0.5, 0.0]},
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        const INDICES: &[u16] = &[
            0, 1, 2
        ];

        let index_buffer = device.create_buffer_init(
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
            .with(Renderable::new(&device)).build();

        let renderer = Renderer::new(&device, &queue, &config);

        Self {
            _instance: instance,
            _adapter: adapter,
            surface,
            device,
            queue,
            config,
            window_size,
            camera,
            camera_controller,
            world,
            renderer,
        }
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.window_size = new_window_size;
            self.config.width = new_window_size.width;
            self.config.height = new_window_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.projection.resize(new_window_size.width, new_window_size.height);
            self.renderer.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
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
        self.renderer.draw(&self.surface, &self.device, &self.queue, &self.world)
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