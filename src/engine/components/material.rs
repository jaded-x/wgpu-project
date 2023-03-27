use egui_inspector::*;
use egui_inspector_derive::EguiInspect;
use specs::{Component, DefaultVecStorage};

#[derive(Component, EguiInspect)]
#[storage(DefaultVecStorage)]
pub struct Material {
    #[inspect(widget = "Slider", min = 0.0, max = 1.0, speed = 0.01)]
    color: cg::Vector3<f32>,    
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: cg::Vector3 { x: 1.0, y: 0.0, z: 1.0}
        }
    }
}

impl Material {
    pub fn new(color: cg::Vector3<f32>) -> Self {
        Self {
            color
        }
    }

    pub fn get_data(&self) -> cg::Vector3<f32> {
        self.color
    }
}