use std::{path::PathBuf, collections::HashMap, sync::{Arc, Mutex}, ffi::OsStr};
use rand::random;
use serde::{Serialize, Deserialize};
use wgpu::util::DeviceExt;
use std::error::Error;

use crate::util::cast_slice;

use super::{texture::Texture, model::Material, gpu::Gpu, renderer::Renderer};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Texture,
    Material,
    Mesh,
    Unknown
}

impl AssetType {
    pub fn from_extension(extension: &OsStr) -> AssetType {
        match extension.to_str().unwrap() {
            "revmat" => AssetType::Material,
            "png" => AssetType::Texture,
            "jpg" => AssetType::Texture,
            "obj" => AssetType::Mesh,
            _ => AssetType::Unknown,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    id: usize,
    file_path: PathBuf,
    pub asset_type: AssetType
}

impl AssetMetadata {
    pub fn get_type(&self) -> AssetType {
        self.asset_type
    }
}

pub struct Registry {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub textures: HashMap<usize, Arc<Texture>>,
    pub materials: HashMap<usize, Arc<Gpu<Material>>>,
    pub metadata: HashMap<usize, AssetMetadata>,
}

impl Registry {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            textures: HashMap::new(),
            materials: HashMap::new(),
            metadata: load_metadata().unwrap(),
        }
    }

    pub fn add(&mut self, file_path: PathBuf) -> usize {
        if self.contains_path(&file_path) {
            return self.get_id(file_path);
        }

        let id = random::<usize>();
        self.metadata.insert(id, AssetMetadata {
            id,
            file_path: file_path.clone(),
            asset_type: AssetType::from_extension(file_path.extension().unwrap())
        });

        self.save_metadata().unwrap();

        id
    }

    fn contains_path(&self, file_path: &PathBuf) -> bool {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == *file_path)
            .map(|(id, _)| *id) {
                Some(_) => true,
                None => false
            }
    }

    fn load_texture(&mut self, id: usize, normal: bool) {
        if let Some(asset) = self.metadata.get(&id) {
            let bytes = std::fs::read(&asset.file_path).expect(&asset.file_path.to_str().unwrap());
            let texture = Texture::from_bytes(&self.device, &self.queue, &bytes, &asset.file_path.to_str().unwrap(), normal).unwrap();

            self.textures.insert(asset.id, Arc::new(texture));
        }
    }

    pub fn get_texture(&mut self, id: usize, normal: bool) -> Option<Arc<Texture>> {
        if !self.textures.contains_key(&id) {
            self.load_texture(id, normal);
        }

        self.textures.get(&id).cloned()
    }

    pub fn get_material(&mut self, id: usize) -> Option<Arc<Gpu<Material>>> {
        if !self.materials.contains_key(&id) {
            self.load_material(id);
        }

        self.materials.get(&id).cloned()
    }

    pub fn get_id(&mut self, file_path: PathBuf) -> usize {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == file_path)
            .map(|(id, _)| *id) {
                Some(id) => id,
                None => self.add(file_path)
            }
    }

    fn load_material(&mut self, id: usize) {
        if let Some(asset) = self.metadata.get(&id).cloned() {
            if !self.materials.contains_key(&id) {
                let material = Arc::new(Mutex::new(Material::load(&asset.file_path)));
                let material_lock = material.lock().unwrap();
                let diffuse_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: cast_slice(&[material_lock.diffuse]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        
                let default = &Texture::default();
                let default_normal = &Texture::default_normal();
        
                let diffuse_texture = { 
                    match material_lock.diffuse_texture.id {
                        Some(id) => {self.get_texture(id, false).unwrap()},
                        None => default.clone(),
                    }
                };
        
                let normal_texture = {
                    match material_lock.normal_texture.id {
                        Some(id) => self.get_texture(id, true).unwrap(),
                        None => default_normal.clone()
                    }
                };
                
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &Renderer::get_material_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: diffuse_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                        },
                    ],
                    label: Some("material_bind_group"),
                });

                drop(material_lock);

                self.materials.insert(asset.id, Arc::new(Gpu::create(material, self.queue.clone(), vec![diffuse_buffer], bind_group)));
            }
        }
    }

    pub fn reload_material(&mut self, id: usize) {
        if let Some(asset) = self.metadata.get(&id).cloned() {
            if self.materials.contains_key(&id) {
                let material = Arc::new(Mutex::new(Material::load(&asset.file_path)));
                let material_lock = material.lock().unwrap();
                let diffuse_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: cast_slice(&[material_lock.diffuse]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        
                let default = &Texture::default();
                let default_normal = &Texture::default_normal();

                let diffuse_texture = { 
                    match material_lock.diffuse_texture.id {
                        Some(id) => {self.get_texture(id, false).unwrap()},
                        None => default.clone(),
                    }
                };
        
                let normal_texture = {
                    match material_lock.normal_texture.id {
                        Some(id) => self.get_texture(id, true).unwrap(),
                        None => default_normal.clone()
                    }
                };
                
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &Renderer::get_material_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: diffuse_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                        },
                    ],
                    label: Some("material_bind_group"),
                });

                drop(material_lock);

                self.materials.insert(asset.id, Arc::new(Gpu::create(material, self.queue.clone(), vec![diffuse_buffer], bind_group)));
            }
        }
    }

    pub fn get_material_from_path(&mut self, file_path: PathBuf) -> Option<Arc<Gpu<Material>>> {
        let id = self.get_id(file_path);
        self.get_material(id)
    }

    fn save_metadata(&self) -> Result<(), Box<dyn Error>> {
        let yaml = serde_yaml::to_string(&self.metadata)?;
        std::fs::write(std::env::current_dir().unwrap().join("registry.yaml"), yaml)?;

        Ok(())
    }
}

fn load_metadata() -> Result<HashMap<usize, AssetMetadata>, Box<dyn Error>> {
    let yaml = std::fs::read_to_string(std::env::current_dir().unwrap().join("registry.yaml"))?;
    let metadata: HashMap<usize, AssetMetadata> = serde_yaml::from_str(&yaml)?;
    Ok(metadata)
}