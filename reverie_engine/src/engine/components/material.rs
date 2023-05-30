
use std::sync::Arc;

use specs::{Component, VecStorage};

use crate::engine::{gpu::Gpu, model::Material, registry::Registry};


#[derive(Component)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub id: usize,
    pub material: Arc<Gpu<Material>>
}

impl MaterialComponent {
    pub fn new(id: usize, registry: &mut Registry) -> Self {
        let material = registry.get_material(id).unwrap();

        Self {
            id,
            material,
        }
    }
}