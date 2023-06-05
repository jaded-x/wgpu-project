use serde::{Serialize, Deserialize};
use specs::{Component, DefaultVecStorage};

#[derive(Clone, Default, Component, Serialize, Deserialize)]
#[storage(DefaultVecStorage)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}