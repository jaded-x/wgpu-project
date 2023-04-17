use specs::{prelude::*, Component};

use wgpu::util::DeviceExt;

use egui_inspector::*;
use egui_inspector_derive::EguiInspect;

use crate::{engine::gpu::Asset, util::cast_slice};

use std::rc::Rc;

#[derive(EguiInspect)]
pub struct TransformData {
    #[inspect(speed = 0.01)]
    position: cg::Vector3<f32>,
    #[inspect(widget = "Slider", min = 0.0, max = 360.0)]
    rotation: cg::Vector3<f32>,
    #[inspect(speed = 0.01)]
    scale: cg::Vector3<f32>,

    #[inspect(hide = true)]
    matrix: cg::Matrix4<f32>,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub data: TransformData,

    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Transform {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let data = TransformData::default();
        
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform_buffer"),
            contents: cast_slice::<cg::Matrix4<f32>>(&[cg::SquareMatrix::identity()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        Self {
            data,
            buffer,
            bind_group,
        }
    }

    pub fn set_position(&mut self, position: cg::Vector3<f32>, queue: &wgpu::Queue) {
        self.data.position = position;
        self.data.update_matrix();
        self.update_buffer(queue);
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, cast_slice(&[self.data.matrix]));
    }
}


impl TransformData {
    pub fn new(position: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
            matrix: calculate_transform_matrix(position, rotation, scale),
        }
    }

    pub fn update_matrix(&mut self) {
        let rotation = cg::Matrix4::from_angle_x(cg::Deg(self.rotation.x))
            * cg::Matrix4::from_angle_y(cg::Deg(self.rotation.y))
            * cg::Matrix4::from_angle_z(cg::Deg(self.rotation.z));
        
        self.matrix = cg::Matrix4::from_translation(self.position) * rotation * cg::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

}

impl Default for TransformData {
    fn default() -> Self {
        Self { 
            position: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: cg::Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            matrix: cg::SquareMatrix::identity(),
        }
    }
}

fn calculate_transform_matrix(position: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> cg::Matrix4<f32> {
    let rotation = cg::Matrix4::from_angle_x(cg::Deg(rotation.x))
        * cg::Matrix4::from_angle_y(cg::Deg(rotation.y))
        * cg::Matrix4::from_angle_z(cg::Deg(rotation.z));
    
    cg::Matrix4::from_translation(position) * rotation * cg::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
}