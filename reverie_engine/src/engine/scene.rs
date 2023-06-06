use std::{path::PathBuf, collections::HashMap};

use serde::Deserialize;
use specs::{World, WorldExt, Join, Builder};

use super::{light_manager::LightManager, renderer::Renderer, components::{name::Name, transform::{Transform, DeserializedData, TransformData}, material::MaterialComponent, mesh::Mesh, light::PointLight}, registry::Registry};

pub struct Scene {
    pub path: PathBuf,
    pub world: World,
    pub light_manager: LightManager
}

impl Scene {
    pub fn new(path: PathBuf, registry: &mut Registry, device: &wgpu::Device) -> Self {
        let world = load_scene(&path, registry, device);
        let light_manager = LightManager::new(device, &Renderer::get_light_layout(), &world);

        Self {
            path,
            world,
            light_manager,
        }
    }

    pub fn save_scene(&mut self) {
        let entities = self.world.entities();
        let names = self.world.read_storage::<Name>();
        let transforms = self.world.read_storage::<Transform>();
        let materials = self.world.read_storage::<MaterialComponent>();
        let meshes = self.world.read_storage::<Mesh>();
        let lights = self.world.read_storage::<PointLight>();
        
        let mut s_names = HashMap::new();
        let mut s_transforms = HashMap::new();
        let mut s_materials = HashMap::new();
        let mut s_meshes = HashMap::new();
        let mut s_lights = HashMap::new();

        for entity in entities.join() {
            if let Some(name) = names.get(entity) {
                s_names.insert(entity.id(), name.clone());
            }
            if let Some(transform) = transforms.get(entity) {
                s_transforms.insert(entity.id(), transform.data.clone());
            }
            if let Some(material) = materials.get(entity) {
                s_materials.insert(entity.id(), material.clone());
            }
            if let Some(mesh) = meshes.get(entity) {
                s_meshes.insert(entity.id(), mesh.clone());
            }
            if let Some(light) = lights.get(entity) {
                s_lights.insert(entity.id(), light.clone());
            }
        }

        let yaml_names = serde_yaml::to_string(&s_names).unwrap();
        let yaml_transforms = serde_yaml::to_string(&s_transforms).unwrap();
        let yaml_materials = serde_yaml::to_string(&s_materials).unwrap();
        let yaml_meshes = serde_yaml::to_string(&s_meshes).unwrap();
        let yaml_lights = serde_yaml::to_string(&s_lights).unwrap();

        let yaml = format!(
            "# Names\n{}\n\n# Transforms\n{}\n\n# Materials\n{}\n\n# Meshes\n{}\n\n# Lights\n{}",
            yaml_names, yaml_transforms, yaml_materials, yaml_meshes, yaml_lights
        );
        
        std::fs::write(self.path.clone(), yaml).unwrap();
    } 

    pub fn load_scene(&mut self, path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) {
        self.world = load_scene(path, registry, device);
    
        self.light_manager = LightManager::new(device, &Renderer::get_light_layout(), &self.world);
        self.path = path.clone();
    }
}

fn load_scene(path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) -> World {
    let yaml = std::fs::read_to_string(path).unwrap();
    if yaml == "" {
        let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Name>();
        world.register::<PointLight>();
        world.create_entity().with(Transform::new(TransformData::default(), device, &Renderer::get_transform_layout())).with(Name::new("Light")).with(PointLight::new([0.0, 0.0, 0.0])).build();
        return world;
    }
    let sections: Vec<&str> = yaml.split("\n\n").collect();

    let s_names: HashMap<u32, Name> = serde_yaml::from_str(sections[0]).unwrap();
    let s_transforms: HashMap<u32, DeserializedData> = serde_yaml::from_str(sections[1]).unwrap();
    let s_materials: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[2]).unwrap();
    let s_meshes: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[3]).unwrap();
    let s_lights: HashMap<u32, PointLight> = serde_yaml::from_str(sections[4]).unwrap();

    let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Name>();
        world.register::<PointLight>();

    for id in 0..s_names.len() as u32 {
        let mut entity = world.create_entity();

        if let Some(name) = s_names.get(&id) {
            entity = entity.with(name.clone());
        }
        if let Some(transform) = s_transforms.get(&id) {
            entity = entity.with(Transform::new(TransformData::new(transform.position, transform.rotation, transform.scale), device, &Renderer::get_transform_layout()))
        }
        if let Some(material) = s_materials.get(&id) {
            entity = entity.with(MaterialComponent::new(material.id, registry))
        }
        if let Some(mesh) = s_meshes.get(&id) {
            entity = entity.with(Mesh::new(mesh.id, registry))
        }
        if let Some(light) = s_lights.get(&id) {
            entity = entity.with(light.clone())
        }

        entity.build();
    }

    world
}

#[derive(Deserialize)]
struct DeserializedId {
    id: usize
}