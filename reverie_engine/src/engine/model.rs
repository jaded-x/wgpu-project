use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use wgpu::util::DeviceExt;

use imgui_inspector_derive::ImguiInspect;
use imgui_inspector::*;

use crate::util::cast_slice;

use super::gpu::{Asset, Gpu};

use super::registry::Registry;
use super::texture::Texture;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}

pub struct Model {
    pub meshes: Vec<Arc<Mesh>>,
    pub materials: Vec<Gpu<Material>>,
}

#[derive(Serialize, Deserialize)]
pub struct TextureId {
    id: Option<usize>,
}

impl TextureId {
    pub fn new(id: Option<usize>) -> Self {
        Self {
            id
        }
    }
}

impl InspectTexture for TextureId {
    fn inspect_texture<'a>(&mut self, ui: &'a imgui::Ui, label: &str) -> bool {
        let mut result = false;
        
        ui.button("test");
        match ui.drag_drop_target() {
            Some(target) => {
                match target.accept_payload::<Option<usize>, _>("texture", imgui::DragDropFlags::empty()) {
                    Some(Ok(payload_data)) => {
                        self.id = payload_data.data;
                        result = true;
                        dbg!(payload_data.data);
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


#[derive(ImguiInspect, Serialize, Deserialize)]
pub struct Material {
    #[inspect(widget = "color")]
    pub diffuse: [f32; 3],
    #[inspect(widget = "texture")]
    pub diffuse_texture: TextureId,
    #[inspect(hide = true)]
    pub normal_texture: TextureId,
}

impl Material {
    pub fn new(diffuse: [f32; 3], diffuse_texture: Option<usize>, normal_texture: Option<usize>) -> Self {
        Self {
            diffuse,
            diffuse_texture: TextureId::new(diffuse_texture),
            normal_texture: TextureId::new(normal_texture),
        }
    }

    pub fn create(path: &PathBuf, name: &str) -> Self {
        let file_name = format!("{}{}", name, ".revmat");

        match std::fs::File::create(path.join(&file_name)) {
            Err(e) => eprintln!("Failed to create file: {}", e),
            _ => {}
        }

        let material = Self {
            diffuse: [1.0, 1.0, 1.0],
            diffuse_texture: TextureId::new(None),
            normal_texture: TextureId::new(None),
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
        std::fs::write(path, yaml).unwrap();
    }
}

impl Gpu<Material> {
    pub fn update_diffuse_buffer(&self, diffuse: [f32; 3]) {
        self.update_buffer(0, cast_slice(&[diffuse]));
    }
} 

impl Asset for Material {
    fn load(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>, registry: &mut Registry) -> (Vec<wgpu::Buffer>, wgpu::BindGroup) {
        let diffuse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: cast_slice(&[self.diffuse]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let default = &Texture::default();
        let default_normal = &Texture::default_normal();

        let diffuse_texture = { 
            match self.diffuse_texture.id {
                Some(id) => {registry.get_texture(id, false).unwrap()},
                None => default.clone(),
            }
        };

        let normal_texture = {
            match self.normal_texture.id {
                Some(id) => registry.get_texture(id, true).unwrap(),
                None => default_normal.clone()
            }
        };
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
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
        
        (vec![diffuse_buffer], bind_group)
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub element_count: u32,
    pub material: usize,
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Gpu<Material>,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Gpu<Material>,
        instances: Range<u32>,
    );

    // fn draw_model(
    //     &mut self,
    //     model: &'a Model,
    // );
    // fn draw_model_instanced(
    //     &mut self,
    //     model: &'a Model,
    //     instances: Range<u32>,
    // );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> where 'b: 'a {
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Gpu<Material>,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Gpu<Material>,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(2, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.element_count, 0, instances);
    }

    // fn draw_model(
    //     &mut self,
    //     model: &'b Model,
    // ) {
    //     self.draw_model_instanced(model, 0..1);
    // }

    // fn draw_model_instanced(
    //     &mut self,
    //     model: &'b Model,
    //     instances: Range<u32>,
    // ) {
    //     for mesh in &model.meshes {
    //         let material = &model.materials[mesh.material];
    //         self.draw_mesh_instanced(mesh, material, instances.clone());
    //     }
    // }

}