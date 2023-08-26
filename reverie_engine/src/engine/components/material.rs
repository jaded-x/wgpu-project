use std::sync::Arc;

use serde::Serialize;
use specs::{Component, VecStorage};

use crate::engine::{gpu::Gpu, registry::Registry, asset::material::Material};

use super::{ComponentDefault, TypeName};

#[derive(Clone, Component, Serialize)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub id: usize,
    #[serde(skip)]
    pub material: Arc<Gpu<Material>>,
    #[serde(skip)]
    pub is_loaded: bool,
}

impl MaterialComponent {
    pub fn new(id: usize, registry: &mut Registry) -> Self {
        let material = registry.get_material(id);

        Self {
            id,
            material,
            is_loaded: false
        }
    }
}

impl ComponentDefault for MaterialComponent {
    fn default(_device: &wgpu::Device, registry: &mut Registry) -> Self {
        let material = registry.get_material(1);

        Self {
            id: 1,
            material,
            is_loaded: true,
        }
    }
}

impl TypeName for MaterialComponent {
    fn type_name() -> &'static str {
        "material"
    }
}