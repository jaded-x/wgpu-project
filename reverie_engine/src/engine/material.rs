use std::{path::PathBuf, sync::Arc};

use imgui_inspector_derive::ImguiInspect;
use imgui_inspector::*;
use serde::{Serialize, Deserialize};

use crate::util::cast_slice;

use super::{registry::AssetType, gpu::{Gpu, Asset}, texture::Texture};

#[derive(Serialize, Deserialize)]
pub struct TextureId {
    pub id: Option<usize>,
    #[serde(skip)]
    pub texture: Option<Arc<Texture>>
}

impl TextureId {
    pub fn new(id: Option<usize>) -> Self {
        Self {
            id,
            texture: None,
        }
    }
}

impl InspectTexture for TextureId {
    fn inspect_texture<'a>(&mut self, ui: &'a imgui::Ui, label: &str) -> bool {
        let mut result = false;
        match self.id {
            Some(id) => imgui::Image::new(imgui::TextureId::new(id), [32.0, 32.0]).border_col([1.0, 1.0, 1.0, 1.0]).build(ui),
            None => imgui::Image::new(imgui::TextureId::new(5), [32.0, 32.0]).border_col([1.0, 1.0, 1.0, 1.0]).build(ui),
        } 
        
        match ui.drag_drop_target() {
            Some(target) => {
                match target.accept_payload::<Option<usize>, _>(AssetType::Texture.to_string(), imgui::DragDropFlags::empty()) {
                    Some(Ok(payload_data)) => {
                        self.id = payload_data.data;
                        result = true;
                    },
                    Some(Err(e)) => {
                        println!("{}", e);
                    },
                    _ => {},
                }
            },
            _ => {},
        }
        ui.same_line();
        ui.text(label);

        result
    }
}

#[repr(align(16))]
#[derive(ImguiInspect, Serialize, Deserialize, Clone, Copy)]
pub struct PBR {
    #[inspect(widget = "color")]
    pub albedo: [f32; 3],
    #[inspect(widget = "drag", min = 0.0, max = 1.0, speed = 0.05)]
    pub metallic: f32,
    #[inspect(widget = "drag", min = 0.0, max = 1.0, speed = 0.05)]
    pub roughness: f32,
    #[inspect(widget = "drag", min = 0.0, max = 1.0, speed = 0.05)]
    pub ao: f32,
}

impl Default for PBR {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0],
            metallic: 0.5,
            roughness: 0.5,
            ao: 0.5,
        }
    }
}


#[derive(ImguiInspect, Serialize, Deserialize)]
pub struct Material {
    #[inspect(hide = true)]
    pub floats: PBR,
    #[inspect(widget = "texture")]
    pub albedo_map: TextureId,
    #[inspect(widget = "texture")]
    pub normal_map: TextureId,
    #[inspect(widget = "texture")]
    pub metallic_map: TextureId,
    #[inspect(widget = "texture")]
    pub roughness_map: TextureId,
    #[inspect(widget = "texture")]
    pub ao_map: TextureId,
}

impl Material {
    pub fn new(albedo: Option<usize>, normal: Option<usize>, metallic: Option<usize>, roughness: Option<usize>, ao: Option<usize>) -> Self {
        Self {
            floats: PBR::default(),
            albedo_map: TextureId::new(albedo),
            normal_map: TextureId::new(normal),
            metallic_map: TextureId::new(metallic),
            roughness_map: TextureId::new(roughness),
            ao_map: TextureId::new(ao),
        }
    }

    pub fn create(path: &PathBuf, name: &str) -> Self {
        let file_name = format!("{}{}", name, ".revmat");

        match std::fs::File::create(path.join(&file_name)) {
            Err(e) => eprintln!("Failed to create file: {}", e),
            _ => {}
        }

        let material = Self {
            floats: PBR::default(),
            albedo_map: TextureId::new(None),
            normal_map: TextureId::new(None),
            metallic_map: TextureId::new(None),
            roughness_map: TextureId::new(None),
            ao_map: TextureId::new(None),
        };

        let yaml = serde_yaml::to_string(&material).unwrap();
        std::fs::write(path.join(&file_name), yaml).unwrap();
        
        material
    }
    
    pub fn load(path: &PathBuf) -> Self {
        let yaml = std::fs::read_to_string(path).unwrap();
        let material: Material = serde_yaml::from_str(&yaml).unwrap();

        material
    }

    pub fn save(&self, path: &PathBuf) {
        let yaml = serde_yaml::to_string(self).unwrap();
        if let Some(extension) = path.extension() {
            if AssetType::from_extension(extension) == AssetType::Material {
                std::fs::write(path, yaml).unwrap();
            }
        }
    }
}

impl Gpu<Material> {
    pub fn update_floats(&self, floats: PBR) {
        self.update_buffer(0, cast_slice(&[floats]));
    }
}

impl Asset for Material {}