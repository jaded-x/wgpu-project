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

    pub fn update_keyboard(&mut self, state: ElementState, key: Option<Key>) {
        match (key, state == ElementState::Pressed) {
            (Some(key), true) => {
                self.pressed_keys.insert(key);
                self.down_keys.insert(key);
            }
            (Some(key), false) => {
                self.down_keys.remove(&key);
            }
            _ => {},
        };
    }

    pub fn update_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match state == ElementState::Pressed {
            true => {
                self.pressed_mouse_buttons.insert(button);
                self.down_mouse_buttons.insert(button);
            }
            false => {
                self.down_mouse_buttons.remove(&button);
            }
        };
    }

    pub fn update_mouse_motion(&mut self, delta: cg::Vector2<f32>) {
        self.cursor_pos += delta;
        self.cursor_delta += delta;
    }

    pub fn update_mouse_wheel(&mut self, delta: cg::Vector2<f32>) {
        self.scroll_delta += delta;
    }
}