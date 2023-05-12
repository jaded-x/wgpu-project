use specs::{*, WorldExt};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

use super::{light::PointLight, components::transform::Transform};

struct LightData {
    _position: Align16<cg::Vector3<f32>>,
    _color: Align16<cg::Vector3<f32>>
}

pub struct LightManager {
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
}

impl LightManager {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, world: &World) -> Self {
        let mut lights = Vec::new();
        let mut light_count: i32 = 0;

        let transform_components = world.read_component::<Transform>();
        let light_components = world.read_component::<PointLight>();

        for (transform, light) in (&transform_components, &light_components).join() {
            let transform_data = transform.get_position();
            let light_data = light.get_color();

            lights.push(LightData {
                _position: Align16(transform_data),
                _color: Align16(light_data)
            });

            light_count += 1;
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("position_buffer"),
            contents: cast_slice(&lights.into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("position_buffer"),
            contents: cast_slice(&[light_count]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });

        Self {
            bind_group,
            buffer,
            count_buffer
        }
    }
}