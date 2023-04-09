use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub mesh_id: usize,
}

impl Mesh {
    pub fn new(mesh_id: usize) -> Self {
        Self {
            mesh_id
        }
    }
}