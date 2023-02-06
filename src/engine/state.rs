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
    context::Context,
    egui::Egui,
    input::InputState,
};

pub struct State {
    pub context: Context,
    renderer: Renderer,
    window: winit::window::Window,

    camera: Camera,
    camera_controller: CameraController,
    world: specs::World,
}

impl State {
    async fn new(window: winit::window::Window) -> Self {
        let context = Context::new(&window).await;
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
            window,
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

        // update buffers
        let mut renderables = self.world.write_storage::<Renderable>();
        for (transform, renderable) in (&transforms, &mut renderables).join()  {
            renderable.update_buffer(&self.context.queue, transform.clone());
        }
    }

    fn render(&mut self, egui: &mut Egui) -> Result<(), wgpu::SurfaceError> {

        // state.context.egui_context.begin_frame(egui_state.take_egui_input(&state.window));
        // state.context.ui.ui(&state.context.egui_context);
        // let egui_output = state.context.egui_context.end_frame();
        // egui_state.handle_platform_output(&window, &state.context.egui_context, egui_output.platform_output);

        let egui_input = egui.state.take_egui_input(&self.window);
        let egui_output = egui.context.run(egui_input, |context| {
            let mut style: egui::Style = (*context.style()).clone();
            context.set_style(style);

            let mut frame = egui::containers::Frame::side_top_panel(&context.style());

            let mut test = 0;

            egui::SidePanel::left("top").frame(frame).show(&context, |ui| {
                ui.add(egui::Slider::new(&mut test, 0..=120).text("hi :)"));
            });
            
        });
        
        let clipped_primitives = egui.context.tessellate(egui_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: self.context.window_size.into(),
            pixels_per_point: egui_winit::native_pixels_per_point(&self.window)
        };

        let output = self.context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
        });

        let meshes = self.world.read_storage::<Mesh>();
        let renderables = self.world.write_storage::<Renderable>();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    }
                })
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.renderer.render_pipeline);

        for (renderable, mesh) in (&renderables, &meshes).join()  {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &renderable.bind_group, &[]);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }

        for (id, image) in egui_output.textures_delta.set {
            egui.renderer.update_texture(&self.context.device, &self.context.queue, id, &image);
        }

        drop(render_pass);

        egui.renderer.update_buffers(&self.context.device, &self.context.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render_pass"),
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

        egui.renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);

        drop(render_pass);

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for id in egui_output.textures_delta.free {
            egui.renderer.free_texture(&id);
        }

        Ok(())

        //self.renderer.draw(&self.context.surface, &self.context.device, &self.context.queue, &self.world)
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

    let mut input = InputState::default();
    let mut state = State::new(window).await;

    let mut egui_state = egui_winit::State::new(&event_loop);
    egui_state.set_pixels_per_point(state.window.scale_factor() as f32);

    let mut egui = Egui::new(&event_loop, &state.context);
    egui.state.set_pixels_per_point(egui_winit::native_pixels_per_point(&state.window));

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
            state.update();

            input.finish_frame();
            match state.render(&mut egui) {
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