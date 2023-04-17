use specs::{prelude::*, Component};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Light {
    color: [f32; 3]
}

impl Light {
    pub fn new(color: [f32; 3]) -> Self {
        Self {
            color,
        }
    }
}
