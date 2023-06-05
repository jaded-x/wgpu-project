
use std::sync::Arc;

use serde::Serialize;
use specs::{Component, VecStorage};

use crate::engine::{gpu::Gpu, model::Material, registry::Registry};


#[derive(Clone, Component, Serialize)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub id: usize,
    #[serde(skip)]
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