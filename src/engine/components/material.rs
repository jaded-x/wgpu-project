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
}