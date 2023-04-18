use specs::{*, WorldExt};

use super::{light::PointLight, components::transform::Transform};

pub struct LightManager {
    pub bind_groups: Vec<wgpu::BindGroup>,
}

impl LightManager {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, world: &World) -> Self {
        let mut lights = Vec::new();

        let transform_components = world.read_component::<Transform>();
        let light_components = world.read_component::<PointLight>();

        for (transform, light) in (&transform_components, &light_components).join() {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transform.buffers.get("position").unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: light.color_buffer.as_entire_binding(),
                    }
                ],
                label: Some("light_bind_group"),
            });

            lights.extend([bind_group]);
        }

        Self {
            bind_groups: lights
        }
    }
}