use std::{path::PathBuf, collections::HashMap, sync::mpsc::channel, thread};

use image::GenericImageView;
use serde::Deserialize;
use specs::{World, WorldExt, Join, Builder};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, res};

use super::{light_manager::LightManager, components::{name::Name, transform::{Transform, DeserializedData, TransformData}, material::MaterialComponent, mesh::Mesh, light::{PointLight, DirectionalLight}}, registry::Registry, texture::Texture, renderer::Renderer, camera::Camera};

pub struct Scene {
    pub path: PathBuf,
    pub world: World,
    pub light_manager: LightManager,
    pub skybox: Option<(Texture, wgpu::Buffer)>,
}

impl Scene {
    pub fn new(path: PathBuf, registry: &mut Registry, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let world = load_world(&path, registry, device);
        let light_manager = LightManager::new(device, &world);
        let skybox = load_skybox(&res("textures/skyboxes/starfield/"), device, queue);

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
        let transforms = self.world.read_storage::<Transform>();
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
            yaml_names, yaml_transforms, yaml_materials, yaml_meshes, yaml_point_lights, yaml_directional_lights
        );
        
        std::fs::write(self.path.clone(), yaml).unwrap();
    } 

    pub fn load_scene(&mut self, path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) {
        self.world = load_world(path, registry, device);
    
        self.light_manager = LightManager::new(device, &self.world);
        self.path = path.clone();
    }

    pub fn create_entity(&mut self, device: &wgpu::Device) {
        self.world.create_entity().with(Name::new("Object")).with(Transform::new(TransformData::default(), device)).build();
    }
}

fn load_world(path: &PathBuf, registry: &mut Registry, device: &wgpu::Device) -> World {
    let yaml = std::fs::read_to_string(path).unwrap();
    if yaml == "" {
        let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Name>();
        world.register::<PointLight>();
        world.register::<DirectionalLight>();
        world.create_entity().with(Transform::new(TransformData::default(), device)).with(Name::new("Light")).with(PointLight::new([0.0, 0.0, 0.0])).build();
        return world;
    }
    let sections: Vec<&str> = yaml.split("\n\n").collect();

    let s_names: HashMap<u32, Name> = serde_yaml::from_str(sections[0]).unwrap();
    let s_transforms: HashMap<u32, DeserializedData> = serde_yaml::from_str(sections[1]).unwrap();
    let s_materials: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[2]).unwrap();
    let s_meshes: HashMap<u32, DeserializedId> = serde_yaml::from_str(sections[3]).unwrap();
    let s_point_lights: HashMap<u32, PointLight> = serde_yaml::from_str(sections[4]).unwrap();
    let s_directional_lights: HashMap<u32, DirectionalLight> = serde_yaml::from_str(sections[5]).unwrap();

    let mut world = specs::World::new();
        world.register::<Transform>();
        world.register::<MaterialComponent>();
        world.register::<Mesh>();
        world.register::<Name>();
        world.register::<PointLight>();
        world.register::<DirectionalLight>();

    for id in 0..s_names.len() as u32 {
        let mut entity = world.create_entity();

        if let Some(name) = s_names.get(&id) {
            entity = entity.with(name.clone());
        }
        if let Some(transform) = s_transforms.get(&id) {
            entity = entity.with(Transform::new(TransformData::new(transform.position, transform.rotation, transform.scale), device))
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

fn load_skybox(dir: &PathBuf, device: &wgpu::Device, queue: &wgpu::Queue) -> (Texture, wgpu::Buffer) {
    let directions = ["left", "right", "up", "down", "front", "back"];

    let (tx, rx) = channel();

    for i in &directions {
        let path = dir.join(format!("{}.png", i));
        let tx = tx.clone();
        let path_clone = path.clone();
        thread::spawn(move || {
            let bytes = std::fs::read(&path_clone).expect(path_clone.to_str().unwrap());
            let img = image::load_from_memory(bytes.as_slice()).unwrap();
            
            tx.send((path_clone, img)).unwrap();
        });
    }

    let mut textures: [Option<image::DynamicImage>; 6] = [None, None, None, None, None, None];

    for _ in 0..6 {
        let (path, img) = rx.recv().unwrap();
        let dir = path.file_stem().unwrap().to_str().unwrap();
        match directions.iter().position(|d| d == &dir) {
            Some(idx) => {
                textures[idx] = Some(img);
            }
            None => {
                println!("Directory not found: {}", dir);
            }
        }
    }

    let textures: [image::DynamicImage; 6] = [
        textures[0].take().unwrap(),
        textures[1].take().unwrap(),
        textures[2].take().unwrap(),
        textures[3].take().unwrap(),
        textures[4].take().unwrap(),
        textures[5].take().unwrap(),
    ];

    let dimensions = textures[0].dimensions();

    let size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 6
    };

    let skybox_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for i in 0..6 {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &skybox_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: i },
            },
            &textures[i as usize].to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );
    }

    let skybox_view = skybox_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("skybox view"),
        format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("cubemap_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: None,
        anisotropy_clamp: None,
        border_color: None
    });

    let vertices: [f32; 108] = [
        -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0
];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("skybox_vertex_buffer"),
        contents: cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });



    (Texture {
        texture: skybox_texture,
        view: skybox_view,
        sampler
    }, vertex_buffer)
}

#[derive(Deserialize)]
struct DeserializedId {
    id: usize
}