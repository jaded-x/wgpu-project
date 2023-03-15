use specs::{prelude::*, Component};

use crate::util::align::Align16;

use egui_inspector::*;
use egui_inspector_derive::EguiInspect;

#[derive(Component, Copy, Clone, PartialEq, EguiInspect)]
#[storage(DefaultVecStorage)]
pub struct Transform {
    #[inspect(speed = 1.0)]
    pub translation: cg::Vector3<f32>,
    #[inspect(speed = 1.0)]
    pub rotation: cg::Vector3<f32>,
    pub scale: cg::Vector3<f32>,
}

impl Transform {
    pub fn new(translation: cg::Vector3<f32>, rotation: cg::Vector3<f32>, scale: cg::Vector3<f32>) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn translation(translation: cg::Vector3<f32>) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn get_transform(&self) -> cg::Matrix4<f32> {
        let rotation = cg::Matrix4::from_angle_x(cg::Deg(self.rotation.x))
            * cg::Matrix4::from_angle_y(cg::Deg(self.rotation.y))
            * cg::Matrix4::from_angle_z(cg::Deg(self.rotation.z));
        
        cg::Matrix4::from_translation(self.translation) * rotation * cg::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn aligned(&self) -> (cg::Vector3<f32>, Align16<cg::Vector3<f32>>, Align16<cg::Vector3<f32>>) {
        (self.translation, Align16(self.rotation), Align16(self.scale))
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { 
            translation: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: cg::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: cg::Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}