use std::sync::{Arc, Mutex};

pub struct Gpu<T: Asset> {
    pub asset: Arc<Mutex<T>>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    queue: Arc<wgpu::Queue>,
}

impl<T: Asset> Gpu<T> {
    pub fn create(asset: Arc<Mutex<T>>, queue: Arc<wgpu::Queue>, buffers: Vec<wgpu::Buffer>, bind_group: wgpu::BindGroup) -> Self {
        Self {
            asset,
            buffers,
            bind_group,
            queue,
        }
    }

    pub fn update_buffer(&self, index: usize, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[index], 0, data)
    }
}

pub trait Asset {}
