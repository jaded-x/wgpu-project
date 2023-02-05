use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window,
};

pub enum WindowEvents {
    Resized {
        width: u32,
        height: u32,
    },
    Keyboard {
        state: ElementState,
        virtual_keycode: Option<VirtualKeyCode>,
    },
    Draw,
}

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        // TODO: Add size
        let event_loop = EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_title("wgpu")
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }

    pub fn run(self, mut callback: impl 'static + FnMut(WindowEvents) -> ()) {
        self.event_loop.run(move |event, _, control_flow| 
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
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
                        WindowEvent::Resized(physical_size) => callback(WindowEvents::Resized {
                            width: physical_size.width,
                            height: physical_size.height,
                        }),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            callback(WindowEvents::Resized {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            })
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    callback(WindowEvents::Draw);
                    // let now = instant::Instant::now();
                    // let dt = now - last_render_time;
                    // last_render_time = now;
                    // state.update(dt);
                    // match state.render() {
                    //     Ok(_) => {}
                    //     Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.window_size),
                    //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    //     Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    // }
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        );
    }
}