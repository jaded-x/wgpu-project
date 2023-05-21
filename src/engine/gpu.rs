use std::sync::Arc;

pub struct Gpu<T: Asset> {
    pub asset: T,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    _layout: Arc<wgpu::BindGroupLayout>,
    _device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl<T: Asset> Gpu<T> {
    pub fn new(asset: T, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, queue: Arc<wgpu::Queue>) -> Self {
        let (buffers, bind_group) = asset.load(device.clone(), layout.clone());

        Self {
            asset,
            buffers,
            bind_group,
            queue,
            _device: device,
            _layout: layout,
        }
    }

    pub fn update_buffer(&mut self, index: usize, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[index], 0, data)
    }
}

pub trait Asset {
    fn load(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
