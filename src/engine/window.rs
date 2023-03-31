use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window,
};

use super::input::InputState;

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_resizable(false)
            .with_title("Reverie")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }

    pub fn run(self, mut callback: impl 'static + FnMut(WindowEvents) -> ()) {
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
            } if window_id == self.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                },
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    callback(WindowEvents::Draw);
                }
                Event::RedrawEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}

pub enum WindowEvents {
    Resized {
        width: u32,
        height: u32,
    },
    Keyboard {
        state: ElementState,
        virtual_keycode: Option<VirtualKeyCode>
    },
    Draw,
}