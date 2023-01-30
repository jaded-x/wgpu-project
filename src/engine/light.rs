use wgpu::util::DeviceExt;

use crate::util::{align::*, cast_slice};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub color: Align16<[f32; 3]>,
}

pub struct Light {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(device: &wgpu::Device, light_bind_group_layout: &wgpu::BindGroupLayout, position: [f32; 3], color: [f32; 3]) -> Self {
        let uniform = LightUniform {
            position,
            color: Align16(color),
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding()
            }],
            label: None,
        });

        Self {
            uniform,
            buffer,
            bind_group
        }
    }
}