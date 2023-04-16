use specs::{prelude::*, Component};

use egui_inspector::*;
use egui_inspector_derive::EguiInspect;

use crate::engine::gpu::Asset;

use std::rc::Rc;

#[derive(Component, Clone, PartialEq, EguiInspect)]
#[storage(DefaultVecStorage)]
pub struct Transform {
    #[inspect(speed = 0.01)]
    position: cg::Vector3<f32>,
    #[inspect(widget = "Slider", min = 0.0, max = 360.0)]
    rotation: cg::Vector3<f32>,
    #[inspect(speed = 0.01)]
    scale: cg::Vector3<f32>,

    #[inspect(hide = true)]
    matrix: cg::Matrix4<f32>,
}

impl Transform {
    pub fn new(position: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
            matrix: calculate_transform_matrix(position, rotation, scale),
        }
    }

    pub fn from_position(position: cg::Vector3<f32>) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn position(&mut self, position: cg::Vector3<f32>) {
        self.position = position;
        self.update_matrix();
    }

    pub fn get_position(&self) -> cg::Vector3<f32> {
        self.position
    }

    pub fn update_matrix(&mut self) {
        let rotation = cg::Matrix4::from_angle_x(cg::Deg(self.rotation.x))
            * cg::Matrix4::from_angle_y(cg::Deg(self.rotation.y))
            * cg::Matrix4::from_angle_z(cg::Deg(self.rotation.z));
        
        self.matrix = cg::Matrix4::from_translation(self.position) * rotation * cg::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn get_matrix(&self) -> cg::Matrix4<f32> {
        self.matrix
    }
}

impl Default for Transform {
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

impl Asset for Transform {
    fn load(&self, device: Rc<wgpu::Device>, layout: Rc<wgpu::BindGroupLayout>) -> (Vec<wgpu::Buffer>, wgpu::BindGroup) {
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<cg::Matrix4<f32>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        (vec![transform_buffer], transform_bind_group)
    }
}