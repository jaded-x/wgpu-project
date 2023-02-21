use specs::{prelude::*, Component};

use crate::util::align::Align16;

#[derive(Component, Copy, Clone, PartialEq)]
#[storage(DefaultVecStorage)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Transform {
    pub fn new(position: cgmath::Vector3<f32>, scale: cgmath::Vector3<f32>) -> Self {
        Self {
            position,
            scale,
        }
    }

    pub fn update(&mut self, position: cgmath::Vector3<f32>, scale: cgmath::Vector3<f32>) {
        self.position = position;
        self.scale = scale;
    }

    pub fn aligned(&self) -> (cgmath::Vector3<f32>, Align16<cgmath::Vector3<f32>>) {
        (self.position, Align16(self.scale))
    }

    pub fn size() -> usize {
        struct Sized {
            _position: cgmath::Vector3<f32>,
            _scale: Align16<cgmath::Vector3<f32>>
        }

        std::mem::size_of::<Sized>()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { 
            position: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: cgmath::Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}