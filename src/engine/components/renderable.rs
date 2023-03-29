use super::{
    transform::Transform,
    material::Material,
};

use super::super::renderer::Renderer;

use crate::util::{cast_slice, align::Align16};
use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub transform_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
    pub color_buffer: wgpu::Buffer,
    pub material_bind_group: wgpu::BindGroup,
}

impl Renderable {
    pub fn new(device: &wgpu::Device, renderer: &Renderer) -> Self {
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<cg::Matrix4<f32>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.transform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        let color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Align16::<cg::Vector3<f32>>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
            transform_buffer,
            transform_bind_group,
            color_buffer,
            material_bind_group,
        }
    } 

    pub fn update_transform_buffer(&mut self, queue: &wgpu::Queue, transform: &Transform) {
        queue.write_buffer(&self.transform_buffer, 0, cast_slice(&[transform.get_matrix()]));
    }

    pub fn update_color_buffer(&mut self, queue: &wgpu::Queue, material: &Material) {
        queue.write_buffer(&self.color_buffer, 0, cast_slice(&[Align16(material.get_color())]));
    }
}