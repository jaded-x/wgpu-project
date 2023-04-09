use egui_inspector_derive::EguiInspect;
use egui_inspector::*;
use specs::{Component, VecStorage};

#[derive(Component, EguiInspect)]
#[storage(VecStorage)]
pub struct Mesh {
    pub mesh_id: usize,
}

impl Mesh {
    pub fn new(mesh_id: usize) -> Self {
        Self {
            mesh_id
        }
    }
}