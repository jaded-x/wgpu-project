use specs::{prelude::*, Component};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

#[derive(Component)]
#[storage(VecStorage)]
pub struct PointLight {
    diffuse_color: [f32; 3],

    position_buffer: wgpu::Buffer,
    color_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl PointLight {
    pub fn new(diffuse_color: [f32; 3], device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let position_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light_position_buffer"),
            size: std::mem::size_of::<cg::Vector3<f32>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }); 
        
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_color_buffer"),
            contents: cast_slice(&[Align16(diffuse_color)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: color_buffer.as_entire_binding(),
                }
            ],
            label: Some("light_bind_group"),
        });

        Self {
            diffuse_color,
            position_buffer,
            color_buffer,
            bind_group,
        }
    }
}
