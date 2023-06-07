use serde::{Serialize, Deserialize};
use specs::{Component, DefaultVecStorage};

use crate::engine::registry::Registry;

use super::{ComponentDefault, TypeName};

#[derive(Clone, Default, Component, Serialize, Deserialize)]
#[storage(DefaultVecStorage)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl ComponentDefault for Name {
    fn default(_device: &wgpu::Device, _registry: &mut Registry) -> Self {
        Self {
            0: "Object".to_string()
        }
    }
}

impl TypeName for Name {
    fn type_name() -> &'static str {
        "name"
    }
}