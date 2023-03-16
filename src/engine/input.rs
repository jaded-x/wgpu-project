use std::collections::HashSet;
use winit::event::*;

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
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.cursor_pos.x += delta.0 as f32;
                    self.cursor_pos.y += delta.1 as f32;
                    self.cursor_delta.x += delta.0 as f32;
                    self.cursor_delta.y += delta.1 as f32;
                }
                DeviceEvent::MouseWheel { delta } => {
                    let delta = match *delta {
                        MouseScrollDelta::PixelDelta(pos) => cg::Vector2::new(pos.x as f32, pos.y as f32),
                        MouseScrollDelta::LineDelta(x, y) => cg::Vector2::new(x, y),
                    };
                    self.scroll_delta += delta;
                }
                _ => return false,
            },
            _ => return false,
        }

        false
    }
}