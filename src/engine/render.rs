use std::{rc::Rc, cell::RefCell};

pub struct Render<T: Asset> {
    pub asset: Rc<RefCell<T>>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    queue: Rc<wgpu::Queue>,
}

impl<T: Asset> Render<T> {
    pub fn new(asset: Rc<RefCell<T>>, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, queue: Rc<wgpu::Queue>) -> Self {
        let (buffers, bind_group) = asset.borrow().load(device, layout);

        Self {
            asset,
            buffers,
            bind_group,
            queue,
        }
    }

    pub fn update_buffer(&mut self, index: usize, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[index], 0, data)
    }
}



pub trait Asset {
    fn load(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
