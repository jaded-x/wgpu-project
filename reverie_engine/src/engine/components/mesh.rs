use std::sync::Arc;

use serde::Serialize;
use specs::{Component, VecStorage};

use crate::engine::{model, registry::Registry};

#[derive(Clone, Component, Serialize)]
#[storage(VecStorage)]
pub struct Mesh {
    pub id: usize,
    #[serde(skip)]
    pub mesh: Arc<model::Mesh>,
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