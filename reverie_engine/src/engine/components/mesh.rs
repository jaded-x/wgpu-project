use std::sync::Arc;

use specs::{Component, VecStorage};

use crate::engine::{model, registry::Registry};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub id: usize,
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