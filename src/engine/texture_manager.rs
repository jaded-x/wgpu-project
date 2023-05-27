use std::{collections::HashMap, path::PathBuf, sync::Arc};

use super::texture::{Texture, self};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub textures: HashMap<PathBuf, Arc<Texture>>,
    pub keys: Vec<PathBuf>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            textures: HashMap::new(),
            keys: Vec::new(),
        }
    }
    
    pub fn load_texture(&mut self, path: &PathBuf) -> Arc<texture::Texture> {
        if self.textures.contains_key(path) { 
            return self.get_texture(path);
        }

        let bytes = std::fs::read(path).expect(path.to_str().unwrap());
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, &bytes, path.to_str().unwrap(), false).unwrap();
    
        self.textures.insert(path.clone(), Arc::new(texture));
        self.keys.push(path.clone());

        self.get_texture(path)
    }

    pub fn get_texture(&mut self, path: &PathBuf) -> Arc<texture::Texture> {
        if let Some(texture) = self.textures.get(path) {
            return texture.clone();
        }

        self.load_texture(path)
    }

    pub fn load_normal_texture(&mut self, path: &PathBuf) -> Arc<texture::Texture> {
        if self.textures.contains_key(path) { 
            return self.get_texture(path);
        }
        let bytes = std::fs::read(&path).unwrap();
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, &bytes, path.to_str().unwrap(), true).unwrap();
    
        self.textures.insert(path.clone(), Arc::new(texture));
        self.keys.push(path.clone());

        self.get_texture(path)
    }

    pub fn get_texture_by_index(&self, index: usize) -> Arc<texture::Texture> {
        self.keys.get(index).and_then(|key| self.textures.get(key).cloned()).unwrap().clone()
    }

    pub fn get_index(&mut self, path: &PathBuf) -> Option<usize> {
        self.keys.iter().position(|x| x == path)
    }

    pub fn update_location(&mut self, old_path: PathBuf, new_path: PathBuf) {
        if let Some(texture) = self.textures.remove(&old_path).clone() {
            self.textures.insert(new_path, texture);
        }
    }
}