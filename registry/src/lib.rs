pub mod texture;

use std::{path::PathBuf, collections::HashMap, sync::Arc};
use rand::random;
use serde::{Serialize, Deserialize};
use std::error::Error;

use crate::texture::Texture;

#[derive(Serialize, Deserialize)]
enum AssetType {
    Texture,
    Material,
}

#[derive(Serialize, Deserialize)]
struct AssetMetadata {
    id: usize,
    file_path: PathBuf,
    asset_type: AssetType
}

pub struct Registry {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashMap<usize, Arc<Texture>>,
    metadata: HashMap<usize, AssetMetadata>,
}

impl Registry {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            textures: HashMap::new(),
            metadata: load_metadata().unwrap(),
        }
    }

    pub fn add_texture(&mut self, file_path: PathBuf) -> usize {
        if self.is_file_path(&file_path) {
            return self.get_id(file_path);
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

    pub fn get_id(&mut self, file_path: PathBuf) -> usize {
        match self.metadata.iter()
            .find(|(_, asset)| asset.file_path == file_path)
            .map(|(id, _)| *id) {
                Some(id) => id,
                None => self.add_texture(file_path)
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