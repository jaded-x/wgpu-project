use std::{sync::mpsc::channel, path::PathBuf};

use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::util::cast_slice;

use super::super::{camera::Camera, renderer::Renderer};
use crate::engine::asset::texture::Texture;

pub struct Skybox {
    pub texture: Texture,
    pub vertex_buffer: wgpu::Buffer,
    proj_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup
}

impl Skybox {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera, dir: &PathBuf) -> Self {
        let directions = ["left", "right", "up", "down", "front", "back"];

        let (tx, rx) = channel();

        for i in &directions {
            let path = dir.join(format!("{}.png", i));
            let tx = tx.clone();
            let path_clone = path.clone();
            std::thread::spawn(move || {
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

        let camera_view = camera.calc_matrix();
        let rotation = cg::Matrix3::new(
            camera_view.x.x, camera_view.x.y, camera_view.x.z,
            camera_view.y.x, camera_view.y.y, camera_view.y.z,
            camera_view.z.x, camera_view.z.y, camera_view.z.z,
        );
        let view_mat4 = cg::Matrix4::from(rotation);
        let proj = camera.projection.calc_matrix();
        let proj_view = proj * view_mat4;

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("skybox_index_buffer"),
            contents: cast_slice(&[proj_view]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_skybox_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&skybox_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: proj_buffer.as_entire_binding(),
                }
            ],
            label: Some("skybox_bind_group"),
        });

        Self {
            texture: Texture {
                texture: skybox_texture,
                view: skybox_view,
                sampler,
            },
            vertex_buffer,
            proj_buffer,
            bind_group,
        }
    }

    pub fn update_projection(&self, camera: &Camera, queue: &wgpu::Queue) {
        let camera_view = camera.calc_matrix();
        let rotation = cg::Matrix3::new(
            camera_view.x.x, camera_view.x.y, camera_view.x.z,
            camera_view.y.x, camera_view.y.y, camera_view.y.z,
            camera_view.z.x, camera_view.z.y, camera_view.z.z,
        );
        let view_mat4 = cg::Matrix4::from(rotation);
        let proj = camera.projection.calc_matrix();
        let proj_view = proj * view_mat4;

        queue.write_buffer(&self.proj_buffer, 0, cast_slice(&[proj_view]));
    }
}
