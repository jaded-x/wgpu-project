use specs::{Component, DefaultVecStorage};

#[derive(Default, Component)]
#[storage(DefaultVecStorage)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}