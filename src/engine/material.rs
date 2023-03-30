use wgpu::util::DeviceExt;

use super::{
    texture,
    renderer::Renderer,
};
use crate::util::{align::Align16, cast_slice};
use std::sync::Arc;

pub struct Material {
    pub color: cg::Vector3<f32>,
    pub texture: Option<texture::Texture>,

    color_buffer: wgpu::Buffer,
    pub material_bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn create(device: &wgpu::Device, renderer: &Renderer) -> Arc<Self> {
        let color = cg::vec3(1.0, 1.0, 1.0);

        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: cast_slice(&[Align16(color)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color_buffer.as_entire_binding()
                },
            ],
            label: None,
        });

        Arc::from(Self {
            color,
            texture: None,
            color_buffer,
            material_bind_group,
        })
    }

    pub fn update_color_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.color_buffer, 0, cast_slice(&[self.color]));
    }
}