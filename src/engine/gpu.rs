use std::sync::{Arc, Mutex};

pub struct Gpu<T: Asset> {
    pub asset: Arc<Mutex<T>>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    queue: Arc<wgpu::Queue>,
    manager_index: Option<usize>,
}

impl<T: Asset> Gpu<T> {
    pub fn new(asset: Arc<Mutex<T>>, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, queue: Arc<wgpu::Queue>) -> Self {
        let (buffers, bind_group) = asset.lock().unwrap().load(device.clone(), layout.clone());

        Self {
            asset,
            buffers,
            bind_group,
            queue,
            manager_index: None,
        }
    }

    pub fn update_buffer(&mut self, index: usize, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[index], 0, data)
    }

    pub fn set_manager_index(&mut self, index: usize) {
        self.manager_index = Some(index);
    }

    pub fn get_manager_index(&self) -> Option<usize> {
        self.manager_index
    }
}

pub trait Asset {
    fn load(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
