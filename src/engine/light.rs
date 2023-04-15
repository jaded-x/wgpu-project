use specs::{prelude::*, Component};
use wgpu::util::DeviceExt;

use crate::util::{align::*, cast_slice};

use super::{gpu::{Gpu, Asset}, components::transform::Transform};

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

pub struct LightSource {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

// impl LightSource {
//     pub fn new(world: &specs::World, entity: Entity, device: &wgpu::Device) -> Self {
//         let light_transform = world.read_component::<Transform>();
        

        
//     }
// }

