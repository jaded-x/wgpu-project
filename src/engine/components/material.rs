use specs::{Component, VecStorage};

use crate::engine::material::Material;
use std::sync::Arc;

#[derive(Component)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub material: Arc<Material>,
}

impl MaterialComponent {
    pub fn new(material: Arc<Material>) -> Self {
        Self {
            material
        }
    }

    pub fn get_material_bind_group(&self) -> &wgpu::BindGroup {
        &self.material.material_bind_group
    }

    pub fn get_texture_bind_group(&self) -> &wgpu::BindGroup {
        &self.material.texture_bind_group.as_ref().unwrap()
    }
}