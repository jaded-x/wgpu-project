use std::{rc::Rc, cell::RefCell};

pub struct Gpu<T: Asset> {
    pub asset: Rc<RefCell<T>>,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    _layout: Rc<wgpu::BindGroupLayout>,
    _device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
}

impl<T: Asset> Gpu<T> {
    pub fn new(asset: Rc<RefCell<T>>, device: Rc<wgpu::Device>, layout: Rc<wgpu::BindGroupLayout>, queue: Rc<wgpu::Queue>) -> Self {
        let (buffers, bind_group) = asset.borrow().load(device.clone(), layout.clone());

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
    fn load(&self, device: Rc<wgpu::Device>, layout: Rc<wgpu::BindGroupLayout>) -> (Vec<wgpu::Buffer>, wgpu::BindGroup);
}
