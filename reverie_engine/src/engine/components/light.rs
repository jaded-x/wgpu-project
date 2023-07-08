use serde::{Serialize, Deserialize};
use specs::{prelude::*, Component};

use imgui_inspector_derive::ImguiInspect;
use imgui_inspector::*;

use crate::engine::registry::Registry;

use super::{ComponentDefault, TypeName};

#[derive(Clone, Component, ImguiInspect, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct PointLight {
    #[inspect(widget = "color")]
    diffuse_color: [f32; 3],
    #[inspect(speed = 0.001)]
    pub bias_min: f32,
    #[inspect(speed = 0.001)]
    pub bias_max: f32,
}

impl PointLight {
    pub fn new(diffuse_color: [f32; 3]) -> Self {
        Self {
            diffuse_color,
            bias_max: 0.0,
            bias_min: 0.0,
        }
    }

    pub fn get_color(&self) -> cg::Vector3<f32> {
        cg::vec3(self.diffuse_color[0], self.diffuse_color[1], self.diffuse_color[2])
    }
}

impl ComponentDefault for PointLight {
    fn default(_device: &wgpu::Device, _registry: &mut Registry) -> Self {
        Self {
            diffuse_color: [1.0, 1.0, 1.0],
            bias_max: 0.0,
            bias_min: 0.0,
        }
    }
}

impl TypeName for PointLight {
    fn type_name() -> &'static str {
        "point_light"
    }
}

#[derive(Clone, Component, ImguiInspect, Serialize, Deserialize)]
pub struct DirectionalLight {
    #[inspect(widget = "custom", speed = 0.05)]
    pub direction: cg::Vector3<f32>,
    #[inspect(widget = "color") ]
    pub color: [f32; 3],
}

impl DirectionalLight {
    pub fn new(direction: cg::Vector3<f32>, color: [f32; 3]) -> Self {
        Self {
            direction,
            color,
        }
    }
}

impl ComponentDefault for DirectionalLight {
    fn default(_device: &wgpu::Device, _registry: &mut Registry) -> Self {
        Self {
            direction: cg::vec3(-0.2, -1.0, -0.3),
            color: [1.0, 1.0, 1.0],
        }
    }
}

impl TypeName for DirectionalLight {
    fn type_name() -> &'static str {
        "directional_light"
    }
}