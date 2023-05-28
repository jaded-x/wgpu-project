use specs::{*, WorldExt};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

use super::components::{light::PointLight, transform::Transform};

#[derive(Clone)]
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

        let transform_components = world.read_component::<Transform>();
        let light_components = world.read_component::<PointLight>();

        let light_count = light_components.count() as i32;

        for (transform, light) in (&transform_components, &light_components).join() {
            let transform_data = transform.get_position();
            let light_data = light.get_color();

            lights.push(LightData {
                _position: Align16(transform_data),
                _color: Align16(light_data)
            });
        }

        let lights_data = lights.clone();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&lights_data.into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
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

    pub fn update_light_position(&self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        queue.write_buffer(&self.buffer, (std::mem::size_of::<LightData>() * index) as u64, cast_slice(&[data]));
    }

    pub fn update_light_data(&self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        queue.write_buffer(&self.buffer, (std::mem::size_of::<LightData>()  * index  + 16) as u64, cast_slice(&[data]));
    }
}