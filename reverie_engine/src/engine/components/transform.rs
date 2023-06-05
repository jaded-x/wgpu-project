use std::collections::HashMap;

use cg::Matrix;
use cg::SquareMatrix;
use serde::Deserialize;
use serde::Serialize;
use specs::{prelude::*, Component};

use wgpu::util::DeviceExt;

use crate::util::align::Align16;
use crate::util::cast_slice;

use imgui_inspector_derive::ImguiInspect;
use imgui_inspector::*;

#[derive(Deserialize)]
pub struct DeserializedData {
    pub position: cg::Vector3<f32>,
    pub rotation: cg::Vector3<f32>,
    pub scale: cg::Vector3<f32>,
}

#[derive(Clone, ImguiInspect, Serialize)]
pub struct TransformData {
    #[inspect(widget = "custom", speed = 0.01)]
    position: cg::Vector3<f32>,
    #[inspect(widget = "custom")]
    rotation: cg::Vector3<f32>,
    #[inspect(widget = "custom", min = 0.001, max = 100.0, speed = 0.01)]
    scale: cg::Vector3<f32>,

    #[inspect(hide = true)]
    #[serde(skip)]
    matrix: cg::Matrix4<f32>,
    #[inspect(hide = true)]
    #[serde(skip)]
    normal_matrix: cg::Matrix4<f32>,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub data: TransformData,

    pub buffers: HashMap<&'static str, wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
}

impl Transform {
    pub fn new(transform: TransformData, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("matrix_buffer"),
            contents: cast_slice(&[transform.matrix, transform.normal_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("position_buffer"),
            contents: cast_slice(&[Align16(transform.position)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        Self {
            data: transform,
            buffers: HashMap::from([("matrix", matrix_buffer), ("position", position_buffer)]),
            bind_group,
        }
    }

    pub fn set_position(&mut self, position: cg::Vector3<f32>, queue: &wgpu::Queue) {
        self.data.position = position;
        self.data.update_matrix();
        self.update_buffers(queue);
    }

    pub fn get_position(&self) -> cg::Vector3<f32> {
        self.data.position
    }

    pub fn update_buffers(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffers.get("matrix").unwrap(), 0, cast_slice(&[self.data.matrix, self.data.normal_matrix]));
        queue.write_buffer(&self.buffers.get("position").unwrap(), 0, cast_slice(&[self.data.position]));
    }
}


impl TransformData {
    pub fn new(position: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> Self {
        
        let matrix = calculate_transform_matrix(position, rotation, scale);

        Self {
            position,
            rotation,
            scale,
            matrix,
            normal_matrix: matrix.invert().unwrap().transpose(),
        }
    }

    pub fn update_matrix(&mut self) {
        let rotation = cg::Matrix4::from_angle_x(cg::Deg(self.rotation.x))
            * cg::Matrix4::from_angle_y(cg::Deg(self.rotation.y))
            * cg::Matrix4::from_angle_z(cg::Deg(self.rotation.z));
        
        self.matrix = cg::Matrix4::from_translation(self.position) * rotation * cg::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.normal_matrix = self.matrix.invert().unwrap().transpose();
    }

}

impl Default for TransformData {
    fn default() -> Self {
        let matrix: cg::Matrix4<f32> = cg::SquareMatrix::identity();

        Self { 
            position: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: cg::Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            matrix: cg::SquareMatrix::identity(),
            normal_matrix: matrix.invert().unwrap().transpose(),
        }
    }
}

fn calculate_transform_matrix(position: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> cg::Matrix4<f32> {
    let rotation = cg::Matrix4::from_angle_x(cg::Deg(rotation.x))
        * cg::Matrix4::from_angle_y(cg::Deg(rotation.y))
        * cg::Matrix4::from_angle_z(cg::Deg(rotation.z));
    
    cg::Matrix4::from_translation(position) * rotation * cg::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
}