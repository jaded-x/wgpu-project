use std::sync::Arc;

use serde::Serialize;
use specs::{Component, VecStorage};

use crate::engine::{asset::model, registry::Registry};

use super::{ComponentDefault, TypeName};

#[derive(Clone, Component, Serialize)]
#[storage(VecStorage)]
pub struct Mesh {
    pub id: usize,
    #[serde(skip)]
    pub mesh: Arc<Vec<model::Mesh>>,
}

impl Mesh {
    pub fn new(id: usize, registry: &mut Registry) -> Self {
        let mesh = registry.get_mesh(id).unwrap();

        Self {
            id,
            mesh
        }
    }
}

impl ComponentDefault for Mesh {
    fn default(_device: &wgpu::Device, registry: &mut Registry) -> Self {
        let mesh = registry.get_mesh(0).unwrap();

        Self {
            id: 0,
            mesh,
        }
    }
}

impl TypeName for Mesh {
    fn type_name() -> &'static str {
        "mesh"
    }
}