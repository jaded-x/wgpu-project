use specs::{prelude::*, Component};

use egui_inspector::*;
use egui_inspector_derive::EguiInspect;

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

    pub fn position(&mut self, position: cg::Vector3<f32>) {
        self.position = position;
        self.update_matrix();
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