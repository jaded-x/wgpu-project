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
}

impl PointLight {
    pub fn new(diffuse_color: [f32; 3]) -> Self {
        Self {
            diffuse_color,
        }
    }

    pub fn get_color(&self) -> cg::Vector3<f32> {
        cg::vec3(self.diffuse_color[0], self.diffuse_color[1], self.diffuse_color[2])
    }
}

impl ComponentDefault for PointLight {
    fn default(_device: &wgpu::Device, _registry: &mut Registry) -> Self {
        Self {
            diffuse_color: [255.0, 255.0, 255.0]
        }
    }
}

impl TypeName for PointLight {
    fn type_name() -> &'static str {
        "point_light"
    }
}