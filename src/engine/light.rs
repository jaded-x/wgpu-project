use wgpu::util::DeviceExt;

use crate::util::{align::*, any_as_u8_slice};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub color: Align16<[f32; 3]>,
}

pub struct Light {
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(device: &wgpu::Device, light_bind_group_layout: &wgpu::BindGroupLayout, position: [f32; 3], color: [f32; 3]) -> Self {
        let light_uniform = LightUniform {
            position,
            color: Align16(color),
        };

        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: unsafe { any_as_u8_slice(&[light_uniform]) },
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding()
            }],
            label: None,
        });

        Self {
            light_uniform,
            light_buffer,
            light_bind_group
        }
    }
}