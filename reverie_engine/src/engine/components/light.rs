use serde::{Serialize, Deserialize};
use specs::{prelude::*, Component};

use imgui_inspector_derive::ImguiInspect;
use imgui_inspector::*;

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
