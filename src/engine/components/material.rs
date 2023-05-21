use std::sync::Arc;

use specs::{Component, VecStorage};

use crate::engine::{model::Material, gpu::Gpu};

#[derive(Component)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub material: Arc<Gpu<Material>>,
}

impl MaterialComponent {
    pub fn new(material: Arc<Gpu<Material>>) -> Self {
        Self {
            material
        }
    }
}