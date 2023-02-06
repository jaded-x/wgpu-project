use std::collections::HashSet;
use winit::event::*;

pub type MouseButton = winit::event::MouseButton;

#[derive(Default)]
pub struct InputState {
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub down_mouse_buttons: HashSet<MouseButton>,
}

impl InputState {
    pub fn left_button_down(&self) -> bool {
        self.down_mouse_buttons.contains(&MouseButton::Left)
    }

    pub fn left_button_pressed(&self) -> bool {
        self.pressed_mouse_buttons.contains(&MouseButton::Left)
    }

    pub fn finish_frame(&mut self) {
        self.pressed_mouse_buttons.clear();
    }

    pub fn update<T>(&mut self, event: &Event<T>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    match state == &ElementState::Pressed {
                        true => {
                            self.pressed_mouse_buttons.insert(*button);
                            self.down_mouse_buttons.insert(*button);
                        }
                        false => {
                            self.down_mouse_buttons.remove(button);
                        }
                    };
                }
                _ => return false,
            },
            _ => return false,
        }

        false
    }
}