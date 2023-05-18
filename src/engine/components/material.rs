use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub material_id: usize,
}

impl MaterialComponent {
    pub fn new(material_id: usize) -> Self {
        Self {
            material_id
        }
    }
}