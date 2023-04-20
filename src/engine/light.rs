use egui_inspector_derive::EguiInspect;
use egui_inspector::*;
use specs::{prelude::*, Component};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

#[derive(Component, EguiInspect)]
#[storage(VecStorage)]
pub struct PointLight {
    #[inspect(widget = "Color")]
    diffuse_color: [f32; 3],

    #[inspect(hide = true)]
    pub color_buffer: wgpu::Buffer,
}

impl PointLight {
    pub fn new(diffuse_color: [f32; 3], device: &wgpu::Device) -> Self {
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_color_buffer"),
            contents: cast_slice(&[Align16(diffuse_color)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            diffuse_color,
            color_buffer,
        }
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.color_buffer, 0, cast_slice(&[Align16(self.diffuse_color)]));
    }
}
