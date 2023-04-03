use egui_inspector::*;
use egui_inspector_derive::EguiInspect;
use specs::{Component, VecStorage};

#[derive(Component, EguiInspect)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub material_id: usize,
}

impl MaterialComponent {
    pub fn new(material_id: usize) -> Self {
        Self {
            material_id
        }
    }
}