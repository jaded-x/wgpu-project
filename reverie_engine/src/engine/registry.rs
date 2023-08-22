use std::{path::PathBuf, collections::HashMap, sync::{Arc, Mutex, mpsc::channel}, ffi::OsStr};
use rand::random;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use wgpu::util::DeviceExt;
use std::error::Error;

use crate::util::cast_slice;

use super::{asset::{texture::Texture, model::Mesh, material::Material}, gpu::Gpu, renderer::Renderer, resources};

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
    pub meshes: HashMap<usize, Arc<Vec<Mesh>>>,
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

    fn load_texture(&self, id: usize, normal: bool) -> Option<(usize, Arc<Texture>, imgui_wgpu::Texture)> {
        if let Some(asset) = self.metadata.get(&id) {
            let (sender, receiver) = channel();
            let imgui_renderer = self.imgui_renderer.clone();
            let device = self.device.clone();
            let queue = self.queue.clone();
            let file_path = asset.file_path.clone();
            let bytes = std::fs::read(&asset.file_path).expect(&asset.file_path.to_str().unwrap());
            let img = image::load_from_memory(bytes.as_slice()).unwrap();
            let imgui_img = img.clone();
            
            std::thread::spawn(move || {
                let imgui_texture = create_imgui_texture(imgui_renderer, &imgui_img, file_path.to_str().unwrap(), device, queue, 32, 32);
                sender.send(imgui_texture).unwrap();
            });

            let texture = Texture::from_image(&self.device, &self.queue, &img, Some(asset.file_path.to_str().unwrap()), normal).unwrap();
            
            return Some((asset.id, Arc::new(texture), receiver.recv().unwrap()));
        }
        None
    }

    fn load_mesh(&mut self, id: usize) {
        if let Some(asset) = self.metadata.get(&id) {
            let mesh = &resources::load_mesh(&asset.file_path, &self.device).unwrap();

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
            self.load_material(id, false);
        }

        self.materials.get(&id).cloned()
    }

    pub fn get_mesh(&mut self, id: usize) -> Option<Arc<Vec<Mesh>>> {
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

    pub fn load_material(&mut self, id: usize, is_loaded: bool) {
        if let Some(asset) = self.metadata.get(&id).cloned() {
            if self.materials.contains_key(&id) == is_loaded {
                let material = Arc::new(Mutex::new(Material::load(&asset.file_path)));
                let material_lock = material.lock().unwrap();

                let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: cast_slice(&[material_lock.floats]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let mut bools: Vec<f32> = vec![];

                let texture_option_ids = vec![material_lock.albedo_map.id, material_lock.normal_map.id, material_lock.metallic_map.id, material_lock.roughness_map.id, material_lock.ao_map.id];
                let texture_ids: Vec<usize> = texture_option_ids.into_iter().filter_map(|x| x).collect();
                
                let computed_textures: Vec<_> = texture_ids.into_par_iter().filter_map(|id| {
                    if let Some(_) = self.metadata.get(&id) {
                        let is_normal = self.get_filepath(id).to_str().unwrap().contains("normal");
                        self.load_texture(id, is_normal)
                    } else {
                        None
                    }
                }).collect();

                for (id, texture, imgui_texture) in computed_textures {
                    self.textures.insert(id, texture);
                    self.imgui_renderer.lock().unwrap().textures.replace(imgui::TextureId::new(id), imgui_texture);
                }
        
                let default = &Texture::default();
                let default_normal = &Texture::default_normal();
        
                let albedo_map = { 
                    match material_lock.albedo_map.id {
                        Some(id) => {
                            bools.push(1.0);
                            self.get_texture(id, false).unwrap()
                        },
                        None => {
                            bools.push(0.0);
                            default.clone()
                        },
                    }
                };
        
                let normal_map = {
                    match material_lock.normal_map.id {
                        Some(id) => {
                            self.get_texture(id, true).unwrap()
                        },
                        None => {
                            default_normal.clone()
                        },
                    }
                };

                let metallic_map = {
                    match material_lock.metallic_map.id {
                        Some(id) => {
                            bools.push(1.0);
                            self.get_texture(id, false).unwrap()
                        },
                        None => {
                            bools.push(0.0);
                            default.clone()
                        },
                    }
                };

                let roughness_map = {
                    match material_lock.roughness_map.id {
                        Some(id) => {
                            bools.push(1.0);
                            self.get_texture(id, false).unwrap()
                        },
                        None => {
                            bools.push(0.0);
                            default.clone()
                        },
                    }
                };
                let ao_map = {
                    match material_lock.ao_map.id {
                        Some(id) => {
                            bools.push(1.0);
                            self.get_texture(id, false).unwrap()
                        },
                        None => {
                            bools.push(0.0);
                            default.clone()
                        },
                    }
                };

                let buffer2 = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: cast_slice(bools.as_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &Renderer::get_material_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&albedo_map.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&albedo_map.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&normal_map.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&normal_map.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(&metallic_map.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::Sampler(&metallic_map.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::TextureView(&roughness_map.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::Sampler(&roughness_map.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::TextureView(&ao_map.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::Sampler(&ao_map.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 11,
                            resource: buffer2.as_entire_binding(),
                        }
                    ],
                    label: Some("material_bind_group"),
                });

                drop(material_lock);

                self.materials.insert(asset.id, Arc::new(Gpu::create(material, self.queue.clone(), vec![buffer, buffer2], bind_group)));
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

    pub fn update_filepath(&mut self, id: usize, new_file_path: PathBuf) -> Result<(), Box<dyn Error>> {
        if let Some(metadata) = self.metadata.get_mut(&id) {
            metadata.file_path = new_file_path;
            self.save_metadata()?;
        } else {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Asset ID not found")));
        }

        Ok(())
    }
}

fn load_metadata() -> Result<HashMap<usize, AssetMetadata>, Box<dyn Error>> {
    let yaml = std::fs::read_to_string(std::env::current_dir().unwrap().join("registry.yaml"))?;
    let metadata: HashMap<usize, AssetMetadata> = serde_yaml::from_str(&yaml)?;
    Ok(metadata)
}

fn create_imgui_texture(renderer: Arc<Mutex<imgui_wgpu::Renderer>>, img: &image::DynamicImage, file_name: &str, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, width: u32, height: u32) -> imgui_wgpu::Texture {
    let texture = Texture::from_image(&device, &queue, img, Some(file_name), false).unwrap();
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