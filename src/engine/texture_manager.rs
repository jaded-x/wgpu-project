use std::{collections::HashMap, path::PathBuf, sync::Arc};

use super::texture::{Texture, self};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub textures: HashMap<PathBuf, Arc<Texture>>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            textures: HashMap::new(),
        }
    }
    
    pub fn load_texture(&mut self, path: PathBuf) -> Arc<texture::Texture> {
        if self.textures.contains_key(&path) { 
            return self.get_texture(path);
        }

        let bytes = std::fs::read(&path).expect(&path.to_str().unwrap());
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, &bytes, path.to_str().unwrap(), false).unwrap();
    
        self.textures.insert(path.clone(), Arc::new(texture));
        return self.get_texture(path);
    }

    pub fn get_texture(&mut self, path: PathBuf) -> Arc<texture::Texture> {
        if let Some(texture) = self.textures.get(&path) {
            return texture.clone();
        }

        self.load_texture(path)
    }

    pub fn load_normal_texture(&mut self, path: PathBuf) -> Arc<texture::Texture> {
        if self.textures.contains_key(&path) { 
            return self.get_texture(path);
        }
        let bytes = std::fs::read(&path).unwrap();
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, &bytes, path.to_str().unwrap(), true).unwrap();
    
        self.textures.insert(path.clone(), Arc::new(texture));
        return self.get_texture(path);
    }
}