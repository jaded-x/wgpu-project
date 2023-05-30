use std::{path::PathBuf, collections::HashMap, sync::Arc};
use rand::random;
use serde::{Serialize, Deserialize};
use std::error::Error;

use super::{texture::Texture, model::Material};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Material,
}

#[derive(Serialize, Deserialize)]
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
    pub materials: HashMap<usize, Arc<Material>>,
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

    pub fn add_texture(&mut self, file_path: PathBuf) -> usize {
        if self.is_file_path(&file_path) {
            return self.get_texture_id(file_path);
        }

        let id = random::<usize>();
        self.metadata.insert(id, AssetMetadata {
            id,
            file_path,
            asset_type: AssetType::Texture,
        });

        match self.save_metadata() {
            Err(e) => println!("Failed to save registry metadata: {}", e),
            _ => {}
        }

        id
    }

    pub fn add_material(&mut self, file_path: PathBuf) -> usize {
        if self.is_file_path(&file_path) {
            return self.get_material_id(file_path);
        }

        let id = random::<usize>();
        self.metadata.insert(id, AssetMetadata {
            id,
            file_path,
            asset_type: AssetType::Material,
        });

        match self.save_metadata() {
            Err(e) => println!("Failed to save registry metadata: {}", e),
            _ => {}
        }

        id
    }

    fn is_file_path(&self, file_path: &PathBuf) -> bool {
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

    pub fn get_material(&self, id: usize) -> Option<Arc<Material>> {
        self.materials.get(&id).cloned()
    }

    pub fn get_texture_id(&mut self, file_path: PathBuf) -> usize {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == file_path)
            .map(|(id, _)| *id) {
                Some(id) => id,
                None => self.add_texture(file_path)
            }
    }

    pub fn get_material_id(&mut self, file_path: PathBuf) -> usize {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == file_path)
            .map(|(id, _)| *id) {
                Some(id) => id,
                None => self.add_material(file_path)
            }
    }

    pub fn get_material_id_unchecked(&self, file_path: PathBuf) -> Option<usize> {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == file_path)
            .map(|(id, _)| *id) {
                Some(id) => Some(id),
                None => None
            }
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