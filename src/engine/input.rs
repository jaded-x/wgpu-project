use std::collections::HashSet;
use winit::event::*;

use super::window::WindowEvents;

pub type Key = winit::event::VirtualKeyCode;
pub type MouseButton = winit::event::MouseButton;

pub struct InputState {
    pub pressed_keys: HashSet<Key>,
    pub down_keys: HashSet<Key>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub down_mouse_buttons: HashSet<MouseButton>,
    pub cursor_delta: cg::Vector2<f32>,
    pub cursor_pos: cg::Vector2<f32>,
    pub scroll_delta: cg::Vector2<f32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::default(),
            down_keys: HashSet::default(),
            pressed_mouse_buttons: HashSet::default(),
            down_mouse_buttons: HashSet::default(),
            cursor_delta: cg::Vector2::new(0.0, 0.0),
            cursor_pos: cg::Vector2::new(0.0, 0.0),
            scroll_delta: cg::Vector2::new(0.0, 0.0),
        }
    }
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

    pub fn right_button_down(&self) -> bool {
        self.down_mouse_buttons.contains(&MouseButton::Right)
    }

    pub fn left_button_pressed(&self) -> bool {
        self.pressed_mouse_buttons.contains(&MouseButton::Left)
    }

    pub fn right_button_pressed(&self) -> bool {
        self.pressed_mouse_buttons.contains(&MouseButton::Right)
    }

    pub fn finish_frame(&mut self) {
        self.pressed_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.cursor_delta = cg::Vector2::new(0.0, 0.0);
        self.scroll_delta = cg::Vector2::new(0.0, 0.0);
    }

    // pub fn update<T>(&mut self, event: &Event<T>) -> bool {
    //     match event {
    //         Event::WindowEvent { event, .. } => match event {
    //             WindowEvent::KeyboardInput { input, .. } => {
    //                 match (input.virtual_keycode, input.state == ElementState::Pressed) {
    //                     (Some(key), true) => {
    //                         self.pressed_keys.insert(key);
    //                         self.down_keys.insert(key);
    //                     }
    //                     (Some(key), false) => {
    //                         self.down_keys.remove(&key);
    //                     }
    //                     _ => return false,
    //                 };
    //             }
    //             WindowEvent::MouseInput { state, button, .. } => {
    //                 match state == &ElementState::Pressed {
    //                     true => {
    //                         self.pressed_mouse_buttons.insert(*button);
    //                         self.down_mouse_buttons.insert(*button);
    //                     }
    //                     false => {
    //                         self.down_mouse_buttons.remove(button);
    //                     }
    //                 };
    //             }
    //             _ => return false,
    //         },
    //         Event::DeviceEvent { event, .. } => match event {
    //             DeviceEvent::MouseMotion { delta } => {
    //                 self.cursor_pos.x += delta.0 as f32;
    //                 self.cursor_pos.y += delta.1 as f32;
    //                 self.cursor_delta.x += delta.0 as f32;
    //                 self.cursor_delta.y += delta.1 as f32;
    //             }
    //             DeviceEvent::MouseWheel { delta } => {
    //                 let delta = match *delta {
    //                     MouseScrollDelta::PixelDelta(pos) => cg::Vector2::new(pos.x as f32, pos.y as f32),
    //                     MouseScrollDelta::LineDelta(x, y) => cg::Vector2::new(x, y),
    //                 };
    //                 self.scroll_delta += delta;
    //             }
    //             _ => return false,
    //         },
    //         _ => return false,
    //     }

    //     false
    // }

    // pub fn update(&mut self, event: &InputEvent) -> bool {
    //     match event {
    //         WindowEvent::KeyboardInput(key, true) => {
    //             self.pressed_keys.insert(*key);
    //             self.down_keys.insert(*key);
    //         }
    //         InputEvent::KeyboardInput(key, false) => {
    //             self.down_keys.remove(key);
    //         }
    //         InputEvent::MouseInput(button, true) => {
    //             self.pressed_mouse_buttons.insert(*button);
    //             self.down_mouse_buttons.insert(*button);
    //         }
    //         InputEvent::MouseInput(button, false) => {
    //             self.down_mouse_buttons.remove(button);
    //         }
    //         InputEvent::MouseMotion(delta) => {
    //             self.cursor_pos += *delta;
    //             self.cursor_delta += *delta;
    //         }
    //         InputEvent::MouseWheel(delta) => {
    //             self.scroll_delta += *delta;
    //         }
    //     }
    //     false
    // }

    pub fn update_keyboard(&mut self, state: ElementState, key: Option<Key>) -> bool {
        match (key, state == ElementState::Pressed) {
            (Some(key), true) => {
                self.pressed_keys.insert(key);
                self.down_keys.insert(key);
            }
            (Some(key), false) => {
                self.down_keys.remove(&key);
            }
            _ => return false,
        };
        false
    }

    
}

pub enum InputEvent {
    KeyboardInput(Key, bool),
    MouseInput(MouseButton, bool),
    MouseMotion(cg::Vector2<f32>),
    MouseWheel(cg::Vector2<f32>),
}

impl InputEvent {
    pub fn from_window_event(event: &WindowEvent) -> Option<InputEvent> {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    return Some(InputEvent::KeyboardInput(key, input.state == ElementState::Pressed));
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                return Some(InputEvent::MouseInput(*button, *state == ElementState::Pressed));
            }
            _ => {}
        }
        None
    }

    pub fn from_device_event(event: &DeviceEvent) -> Option<InputEvent> {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                return Some(InputEvent::MouseMotion(cg::Vector2::new(delta.0 as f32, delta.1 as f32)));
            }
            DeviceEvent::MouseWheel { delta } => {
                let delta = match *delta {
                    MouseScrollDelta::PixelDelta(pos) => cg::Vector2::new(pos.x as f32, pos.y as f32),
                    MouseScrollDelta::LineDelta(x, y) => cg::Vector2::new(x, y),
                };
                return Some(InputEvent::MouseWheel(delta));
            }
            _ => {}
        }
        None
    }
}