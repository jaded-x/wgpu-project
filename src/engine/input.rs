use std::collections::HashSet;
use winit::event::*;

pub type Key = winit::event::VirtualKeyCode;
pub type MouseButton = winit::event::MouseButton;

#[derive(Default)]
pub struct InputState {
    pub pressed_keys: HashSet<Key>,
    pub down_keys: HashSet<Key>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub down_mouse_buttons: HashSet<MouseButton>,
}

impl InputState {
    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.down_keys.contains(&key)
    }

    pub fn left_button_down(&self) -> bool {
        self.down_mouse_buttons.contains(&MouseButton::Left)
    }

    pub fn left_button_pressed(&self) -> bool {
        self.pressed_mouse_buttons.contains(&MouseButton::Left)
    }

    pub fn finish_frame(&mut self) {
        self.pressed_keys.clear();
        self.pressed_mouse_buttons.clear();
    }

    pub fn update<T>(&mut self, event: &Event<T>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    match (input.virtual_keycode, input.state == ElementState::Pressed) {
                        (Some(key), true) => {
                            self.pressed_keys.insert(key);
                            self.down_keys.insert(key);
                        }
                        (Some(key), false) => {
                            self.down_keys.remove(&key);
                        }
                        _ => return false,
                    };
                }
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