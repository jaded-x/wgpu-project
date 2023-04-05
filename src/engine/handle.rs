pub struct Handle<T: Asset> {
    pub asset: T,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup
}

impl<T: Asset> Handle<T> {
    pub fn new(asset: T, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let (buffers, bind_group) = asset.load(device, layout);

        Self {
            asset,
            buffers,
            bind_group,
        }
    }
}

pub trait Asset {
    fn load(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}