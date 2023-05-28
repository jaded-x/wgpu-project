use std::sync::Arc;

use registry::Registry;

pub struct Gpu<T: Asset> {
    pub asset: Arc<T>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    queue: Arc<wgpu::Queue>,
}

impl<T: Asset> Gpu<T> {
    pub fn new(asset: Arc<T>, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, queue: Arc<wgpu::Queue>, registry: &mut Registry) -> Self {
        let (buffers, bind_group) = asset.load(device.clone(), layout.clone(), registry);

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

pub trait Asset {
    fn load(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, registry: &mut Registry) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
