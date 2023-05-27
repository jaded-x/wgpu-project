use std::sync::{Arc, Mutex};

use super::model::Material;

pub struct Gpu<T: Asset> {
    pub asset: Arc<Mutex<T>>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    layout:  Arc<wgpu::BindGroupLayout>,
}

impl<T: Asset> Gpu<T> {
    pub fn new(asset: Arc<Mutex<T>>, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, queue: Arc<wgpu::Queue>) -> Self {
        let (buffers, bind_group) = asset.lock().unwrap().load(device.clone(), layout.clone());

        Self {
            asset,
            buffers,
            bind_group,
            device,
            queue,
            layout
        }
    }

    pub fn update_buffer(&self, index: usize, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[index], 0, data)
    }

}

pub trait Asset {
    fn load(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
