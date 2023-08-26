use std::collections::HashMap;
use std::{collections::VecDeque, path::PathBuf};

use async_std::task;
use async_std::sync::Mutex;
use async_std::sync::Arc;

use crate::engine::registry::AssetMetadata;
use crate::engine::registry::Registry;

use super::texture::Texture;

pub struct TextureLoader {
    texture_queue: Arc<Mutex<VecDeque<usize>>>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    imgui_texture_layout: wgpu::BindGroupLayout,
    loaded_textures: Vec<(usize, Texture, imgui_wgpu::Texture)>
}

impl TextureLoader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let imgui_texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("imgui-wgpu bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
       
        Self {
            texture_queue: Arc::new(Mutex::new(VecDeque::new())),
            device,
            queue,
            imgui_texture_layout,
            loaded_textures: Vec::new(),
        }
    }

    pub fn add_to_queue(&self, id: usize) {
        let mut texture_queue = task::block_on(self.texture_queue.lock());
        texture_queue.push_back(id);
    }

    pub async fn load_textures(&mut self, metadata: HashMap<usize, AssetMetadata>) {
        while let Some(id) = self.next_texture().await {
            if let Some(asset) = metadata.get(&id) {
                let texture = self.load_texture(asset).await;
                self.loaded_textures.push((id, texture.0, texture.1))
            }
        }
    }

    async fn next_texture(&self) -> Option<usize> {
        let mut texture_queue = self.texture_queue.lock().await;
        texture_queue.pop_front()
    }

    async fn load_texture(&self, asset: &AssetMetadata) -> (Texture, imgui_wgpu::Texture) {
        let device = self.device.clone();
        let queue = self.queue.clone();
        let file_path = asset.file_path.clone();
        let bytes = std::fs::read(&asset.file_path).expect(&asset.file_path.to_str().unwrap());
        let img = image::load_from_memory(bytes.as_slice()).unwrap();
        let imgui_img = img.clone();
        
        let imgui_texture = create_imgui_texture(&self.imgui_texture_layout, &imgui_img, file_path.to_str().unwrap(), device, queue, 64, 64).await;

        // std::thread::spawn(move || {
        //     let imgui_texture = create_imgui_texture(imgui_renderer, &imgui_img, file_path.to_str().unwrap(), device, queue, 64, 64);
        //     sender.send(imgui_texture).unwrap();
        // });

        let texture = Texture::from_image(&self.device, &self.queue, &img, Some(asset.file_path.to_str().unwrap()), false).unwrap();
        //self.textures.insert(id, Arc::new(texture));

        (texture, imgui_texture)
    }
}

async fn create_imgui_texture(layout: &wgpu::BindGroupLayout, img: &image::DynamicImage, file_name: &str, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, width: u32, height: u32) -> imgui_wgpu::Texture {
    
    let texture = Texture::from_image(&device, &queue, img, Some(file_name), false).unwrap();
    let tex = Arc::new(texture.texture);
    let view = Arc::new(texture.view);
    let bind_group = {
        let config = &imgui_wgpu::RawTextureConfig {
            label: Some("raw texture config"),
            sampler_desc: wgpu::SamplerDescriptor {
                ..Default::default()
            }
        };

        let sampler = device.create_sampler(&config.sampler_desc);

        Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: config.label,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }))
    };

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    imgui_wgpu::Texture::create(tex, view, bind_group, size)
}

// UPDATE OBJECT TEXTURE WHEN IT IS LOADED