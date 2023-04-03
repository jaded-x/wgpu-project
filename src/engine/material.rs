use wgpu::util::DeviceExt;

use super::{
    texture,
    renderer::Renderer,
};
use crate::util::{align::Align16, cast_slice};

pub struct Material {
    pub color: cg::Vector3<f32>,
    pub texture: Option<texture::Texture>,

    color_buffer: wgpu::Buffer,
    pub material_bind_group: wgpu::BindGroup,
    pub texture_bind_group: Option<wgpu::BindGroup>,
}

impl Material {
    pub fn new(color: cg::Vector3<f32>, device: &wgpu::Device, renderer: &Renderer) -> Self {
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

        Self {
            color,
            texture: None,
            color_buffer,
            material_bind_group,
            texture_bind_group: None,
        }
    }

    pub fn set_texture(&mut self, texture: texture::Texture, device: &wgpu::Device, renderer: &Renderer) {
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: None,
        });

        self.texture = Some(texture);
        self.texture_bind_group = Some(texture_bind_group);
    }

    pub fn update_color_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.color_buffer, 0, cast_slice(&[self.color]));
    }
}