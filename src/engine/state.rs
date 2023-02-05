use egui_wgpu::renderer::ScreenDescriptor;
use wgpu::util::DeviceExt;
use winit::event::*;
use specs::prelude::*;

use crate::util::cast_slice;

use super::{
    texture::Texture,
    camera::{Camera, CameraController, Projection},
    components::{
        mesh::{Vert, Mesh},
        transform::Transform,
        renderable::Renderable,
    }, 
    window::{Window, WindowEvents}, 
    renderer::{Renderer, Pass},
    context::Context
};

pub struct State {
    pub context: Context,
    renderer: Renderer,

    camera: Camera,
    camera_controller: CameraController,
    world: specs::World,
}

impl State {
    async fn new(window: &winit::window::Window) -> Self {
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
            if transform.position.x >= 1.0 { transform.position.x = -1.0 }
            transform.position.x = transform.position.x + 0.01;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        

        self.renderer.draw(&self.context.surface, &self.context.device, &self.context.queue, &self.world)
    }


}

use winit::{
    event_loop::{ControlFlow, EventLoop},
};

pub async fn run() {
    env_logger::init();
    //let window = Window::new();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("wgpu")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut egui_state = egui_winit::State::new(&event_loop);
    egui_state.set_pixels_per_point(window.scale_factor() as f32);



    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    _ => {}
                }
                if egui_state.on_event(&state.context.egui_context, &event).repaint {
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let delta_s = last_render_time.elapsed();
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                state.context.egui_context.begin_frame(egui_state.take_egui_input(&window));
                state.context.ui.ui(&state.context.egui_context);
                let egui_output = state.context.egui_context.end_frame();
                egui_state.handle_platform_output(&window, &state.context.egui_context, egui_output.platform_output);

                state.update();
                //state.render();

                state.context.egui_renderer.update_egui_texture_from_wgpu_texture(&state.context.device, &state.renderer.texture_view, wgpu::FilterMode::Linear, state.context.plot_id);

                let clipped_primitives = state.context.egui_context.tessellate(egui_output.shapes);
                let screen_descriptor = ScreenDescriptor {
                    size_in_pixels: [state.context.config.width, state.context.config.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                let output_frame = state.context.surface.get_current_texture().unwrap();
                let output_view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder")
                });

                let meshes = state.world.read_storage::<Mesh>();
                let transforms = state.world.read_storage::<Transform>();
                let mut renderables = state.world.write_storage::<Renderable>();

                for (transform, renderable) in (&transforms, &mut renderables).join()  {
                    renderable.update_buffer(&state.context.queue, transform.clone());
                }

                for idx in 0..egui_output.textures_delta.set.len() {
                    state.context.egui_renderer.update_texture(&state.context.device, &state.context.queue, egui_output.textures_delta.set[idx].0, &egui_output.textures_delta.set[idx].1);
                }
                for idx in 0..egui_output.textures_delta.free.len() {
                    state.context.egui_renderer.free_texture(&egui_output.textures_delta.free[idx]);
                }
                state.context.egui_renderer.update_buffers(&state.context.device, &state.context.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &output_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&state.renderer.render_pipeline);

                for (renderable, mesh) in (&renderables, &meshes).join()  {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.set_bind_group(0, &renderable.bind_group, &[]);
                    render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }

                state.context.egui_renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);

                drop(render_pass);

                state.context.queue.submit(std::iter::once(encoder.finish()));
                output_frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });

    // window.run(move |event| match event {
    //     WindowEvents::Resized { width, height } => {
    //         state.resize(winit::dpi::PhysicalSize { width, height });
    //     }
    //     WindowEvents::Draw => {
    //         let delta_s = last_render_time.elapsed();
    //         let now = instant::Instant::now();
    //         let dt = now - last_render_time;
    //         last_render_time = now;

    //         state.update();
    //         state.render();
    //     }
    //     WindowEvents::Keyboard {
    //         state,
    //         virtual_keycode,
    //     } => {}
    // });
}