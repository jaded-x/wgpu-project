use super::transform::{Transform, TransformSize};

use crate::util::cast_slice;

pub struct Renderable {
    pub transform_data: Transform,
    pub transform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Renderable {
    pub fn new(device: &wgpu::Device) -> Self {
        let transform = Transform::default();

        let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<TransformSize>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        Self {
            transform_data: transform,
            transform_buffer: buffer,
            bind_group,
        }
    } 

    pub fn update_buffer(&mut self, queue: &wgpu::Queue, data: Transform) {
        if self.transform_data != data {
            self.transform_data = data;
            queue.write_buffer(&self.transform_buffer, 0, cast_slice(&[self.transform_data.aligned()]));
        }
    }
}

// struct UpdateBuffer;

// impl<'a> System<'a> for UpdateBuffer {
//     type SystemData = (ReadStorage<'a, Transform>, ReadStorage<'a, Renderable>);

//     fn run(&mut self, (transforms, renderables): Self::SystemData) {
//         for (transform, renderable) in (&transforms, &renderables).join() {

//             if renderable.transform_data != *transform {
//                 queue.write_buffer(&renderable.transform_buffer, 0, cast_slice(&[transform.aligned()]));
//                 renderable.transform_data = *transform;
//             }
//         }
//     }
// }