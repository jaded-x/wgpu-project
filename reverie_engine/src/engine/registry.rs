use std::{path::{PathBuf, Path}, collections::HashMap, sync::{Arc, Mutex}, ffi::OsStr};
use rand::random;
use serde::{Serialize, Deserialize};
use wgpu::util::DeviceExt;
use std::error::Error;

use crate::util::cast_slice;

use super::{texture::Texture, model::{Material, Mesh}, gpu::Gpu, renderer::Renderer, resources};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Texture,
    Material,
    Mesh,
    Scene,
    Unknown
}

impl AssetType {
    pub fn from_extension(extension: &OsStr) -> AssetType {
        match extension.to_str().unwrap() {
            "revmat" => AssetType::Material,
            "png" => AssetType::Texture,
            "jpg" => AssetType::Texture,
            "obj" => AssetType::Mesh,
            "revscene" => AssetType::Scene,
            _ => AssetType::Unknown,
        }
    }
}

impl ToString for AssetType {
    fn to_string(&self) -> String {
        match self {
            AssetType::Material => String::from("revmat"),
            AssetType::Texture => String::from("texture"),
            AssetType::Mesh => String::from("mesh"),
            AssetType::Scene => String::from("scene"),
            AssetType::Unknown => String::from("unknown"),
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
    imgui_renderer: Arc<Mutex<imgui_wgpu::Renderer>>,
    pub textures: HashMap<usize, Arc<Texture>>,
    pub materials: HashMap<usize, Arc<Gpu<Material>>>,
    pub meshes: HashMap<usize, Arc<Mesh>>,
    pub metadata: HashMap<usize, AssetMetadata>,
}

impl Registry {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, imgui_renderer: Arc<Mutex<imgui_wgpu::Renderer>>) -> Self {
        Self {
            device,
            queue,
            imgui_renderer,
            textures: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
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

            let imgui_texture = create_imgui_texture(&self.imgui_renderer, asset.file_path.to_str().unwrap(), &self.device, &self.queue, 32, 32);
            self.imgui_renderer.lock().unwrap().textures.replace(imgui::TextureId::new(id), imgui_texture);
        }
    }

    fn load_mesh(&mut self, id: usize) {
        if let Some(asset) = self.metadata.get(&id) {
            let mesh = &resources::load_mesh(&asset.file_path, &self.device).unwrap()[0];

            self.meshes.insert(asset.id, mesh.to_owned());
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

    pub fn get_mesh(&mut self, id: usize) -> Option<Arc<Mesh>> {
        if !self.meshes.contains_key(&id) {
            self.load_mesh(id);
        }

        self.meshes.get(&id).cloned()
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

    pub fn get_filepath(&self, id: usize) -> PathBuf {
        self.metadata.get(&id).unwrap().file_path.clone()
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

fn create_imgui_texture(renderer: &Arc<Mutex<imgui_wgpu::Renderer>>, file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> imgui_wgpu::Texture {
    let bytes = std::fs::read(Path::new(file_name)).unwrap();
    let texture = Texture::from_bytes(device, queue, &bytes, file_name, false).unwrap();
    imgui_wgpu::Texture::from_raw_parts(
        &device, 
        &renderer.lock().unwrap(), 
        Arc::new(texture.texture), 
        Arc::new(texture.view), 
        None, 
        Some(&imgui_wgpu::RawTextureConfig {
            label: Some("raw texture config"),
            sampler_desc: wgpu::SamplerDescriptor {
                ..Default::default()
            }
        }), 
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    )
}