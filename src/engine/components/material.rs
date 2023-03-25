use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Material {
    color: cg::Vector4<f32>,    
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: cg::Vector4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 }
        }
    }
}