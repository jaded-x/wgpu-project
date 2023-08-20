use std::{path::PathBuf, collections::HashMap};

use serde::Deserialize;
use specs::{World, WorldExt, Join, Builder};

use crate::util::res;

use super::{
    light_manager::LightManager, 
    components::{
        name::Name, 
        transform::{Transform, DeserializedTransform, TransformComponent}, 
        material::MaterialComponent, 
        mesh::Mesh, 
        light::{PointLight, DirectionalLight},
    }, 
    registry::Registry, 
    camera::Camera, 
    skybox::Skybox
};

pub struct Scene {
    pub path: PathBuf,
    pub world: World,
    pub light_manager: LightManager,
    pub skybox: Option<Skybox>,
}

impl Scene {
    pub fn new(path: PathBuf, registry: &mut Registry, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> Self {
        let world = load_world(&path, registry, device);
        let light_manager = LightManager::new(device, &world);
        let skybox = Skybox::new(device, queue, camera, &res("textures/skyboxes/starfield/"));

        Self {
            path,
            world,
            light_manager,
            skybox: Some(skybox),
        }
    }

    pub fn save_scene(&mut self) {
        let entities = self.world.entities();
        let names = self.world.read_storage::<Name>();
        let transforms = self.world.read_storage::<TransformComponent>();
        let materials = self.world.read_storage::<MaterialComponent>();
        let meshes = self.world.read_storage::<Mesh>();
        let point_lights = self.world.read_storage::<PointLight>();
        let directional_lights = self.world.read_storage::<DirectionalLight>();

        
        let mut s_names = HashMap::new();
        let mut s_transforms = HashMap::new();
        let mut s_materials = HashMap::new();
        let mut s_meshes = HashMap::new();
        let mut s_point_lights = HashMap::new();
        let mut s_directional_lights = HashMap::new();

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
            if let Some(light) = point_lights.get(entity) {
                s_point_lights.insert(entity.id(), light.clone());
            }
            if let Some(light) = directional_lights.get(entity) {
                s_directional_lights.insert(entity.id(), light.clone());
            }
        }

        let yaml_names = serde_yaml::to_string(&s_names).unwrap();
        let yaml_transforms = serde_yaml::to_string(&s_transforms).unwrap();
        let yaml_materials = serde_yaml::to_string(&s_materials).unwrap();
        let yaml_meshes = serde_yaml::to_string(&s_meshes).unwrap();
        let yaml_point_lights = serde_yaml::to_string(&s_point_lights).unwrap();
        let yaml_directional_lights = serde_yaml::to_string(&s_directional_lights).unwrap();

        let yaml = format!(
            "# Names\n{}\n\n# Transforms\n{}\n\n# Materials\n{}\n\n# Meshes\n{}\n\n# Point Lights\n{}\n\n# Directional Lights\n{}",
            yaml_names, yaml_transforms, yaml_materials, yaml_meshes, yaml_point_lights, yaml_directional_lights,
        );
        
        std::fs::write(self.path.clone(), yaml).unwrap();
    } 

    pub fn load_scene(&mut self, path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) {
        self.world = load_world(path, registry, device);
    
        self.light_manager = LightManager::new(device, &self.world);
        self.path = path.clone();
    }

    pub fn create_entity(&mut self, device: &wgpu::Device) {
        self.world.create_entity().with(Name::new("Object")).with(TransformComponent::new(Transform::default(), device)).build();
    }
}

fn load_world(path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) -> World {
    let yaml = std::fs::read_to_string(path).unwrap();
    if yaml == "" {
        let mut world = specs::World::new();
        register_components(&mut world);

        world.create_entity().with(TransformComponent::new(Transform::default(), device)).with(Name::new("Light")).with(PointLight::new([0.0, 0.0, 0.0])).build();
        return world;
    }
    let sections: Vec<&str> = yaml.split("\n\n").collect();

    let s_names: HashMap<u32, Name> = serde_yaml::from_str(sections[0]).unwrap();
    let s_transforms: HashMap<u32, DeserializedTransform> = serde_yaml::from_str(sections[1]).unwrap();
    let s_materials: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[2]).unwrap();
    let s_meshes: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[3]).unwrap();
    let s_point_lights: HashMap<u32, PointLight> = serde_yaml::from_str(sections[4]).unwrap();
    let s_directional_lights: HashMap<u32, DirectionalLight> = serde_yaml::from_str(sections[5]).unwrap();

    let mut world = specs::World::new();
    register_components(&mut world);

    for id in 0..s_names.len() as u32 {
        let mut entity = world.create_entity();

        if let Some(name) = s_names.get(&id) {
            entity = entity.with(name.clone());
        }
        if let Some(transform) = s_transforms.get(&id) {
            entity = entity.with(TransformComponent::new(Transform::new(transform.position, transform.rotation, transform.scale, transform.parent), device))
        }
        if let Some(material) = s_materials.get(&id) {
            entity = entity.with(MaterialComponent::new(material.id, registry))
        }
        if let Some(mesh) = s_meshes.get(&id) {
            entity = entity.with(Mesh::new(mesh.id, registry))
        }
        if let Some(light) = s_point_lights.get(&id) {
            entity = entity.with(light.clone())
        }
        if let Some(light) = s_directional_lights.get(&id) {
            entity = entity.with(light.clone())
        };

        entity.build();
    }

    world
}

#[derive(Deserialize)]
struct DeserializedId {
    id: usize
} 

fn register_components(world: &mut World) {
    world.register::<TransformComponent>();
    world.register::<MaterialComponent>();
    world.register::<Mesh>();
    world.register::<Name>();
    world.register::<PointLight>();
    world.register::<DirectionalLight>();
}