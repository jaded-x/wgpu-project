use specs::{*, WorldExt};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

use super::{components::{light::{PointLight, DirectionalLight}, transform::Transform}, renderer::Renderer};

#[derive(Clone)]
pub struct LightData {
    pub _projection: Align16<[cg::Matrix4<f32>; 6]>,
    pub _position: Align16<cg::Vector3<f32>>,
    pub _color: Align16<cg::Vector3<f32>>,
}

#[derive(Clone)]
struct DirectionalData {
    _direction: Align16<cg::Vector3<f32>>,
    _color: Align16<[f32; 3]>
}

pub struct PointShadow {
    pub views: [wgpu::TextureView; 6],
    pub cube_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_groups: Vec<wgpu::BindGroup>,
}

pub struct LightManager {
    pub point_lights: Vec<LightData>,
    directional_lights: Vec<DirectionalData>,
    pub shadow: PointShadow,
    pub bind_group: wgpu::BindGroup,
    pub point_buffer: wgpu::Buffer,
    pub point_count_buffer: wgpu::Buffer,
    pub directional_buffer: wgpu::Buffer,
    pub directional_count_buffer: wgpu::Buffer,
}

impl LightManager {
    pub fn new(device: &wgpu::Device, world: &World) -> Self {
        let mut point_lights: Vec<LightData> = Vec::new();

        let transform_components = world.read_component::<Transform>();
        let point_light_components = world.read_component::<PointLight>();

        let point_light_count = point_light_components.count() as i32 + 1;

        point_lights.push(LightData {
            _projection: Align16([cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity()]),
            _position: Align16(cg::vec3(0.0, 0.0, 0.0)),
            _color: Align16(cg::vec3(0.0, 0.0, 0.0)),
        });

        let mut g_perspectives = Vec::new();

        for (transform, light) in (&transform_components, &point_light_components).join() {
            let transform_data = transform.get_position();
            let light_data = light.get_color();

            let proj = cg::perspective(cg::Deg(90.0), 1.0, 0.1, 15.0);
            let light_pos = transform_data.clone();
            let centers = vec![light_pos + cg::vec3(-1.0, 0.0, 0.0), light_pos + cg::vec3(1.0, 0.0, 0.0), light_pos + cg::vec3(0.0, -1.0, 0.0), light_pos + cg::vec3(0.0, 1.0, 0.0), light_pos + cg::vec3(0.0, 0.0, -1.0), light_pos + cg::vec3(0.0, 0.0, 1.0)];
            let up_vectors = vec![
                cg::vec3(0.0, -1.0, 0.0),  // Positive X
                cg::vec3(0.0, -1.0, 0.0),  // Negative X
                cg::vec3(0.0, 0.0, 1.0), // Positive Y
                cg::vec3(0.0, 0.0, -1.0), // Negative Y
                cg::vec3(0.0, -1.0, 0.0),  // Positive Z
                cg::vec3(0.0, -1.0, 0.0)  // Negative Z
            ];
            let perspectives = centers.iter().zip(up_vectors.iter()).map(|(center, up)| {
                proj * cg::Matrix4::look_at_rh(cg::point3(light_pos.x, light_pos.y, light_pos.z), cg::point3(center.x, center.y, center.z), *up)
            }).collect::<Vec<_>>();

            point_lights.push(LightData {
                _projection: Align16([perspectives[0], perspectives[1], perspectives[2], perspectives[3], perspectives[4], perspectives[5]]),
                _position: Align16(transform_data),
                _color: Align16(light_data),
            });

            g_perspectives = perspectives;
        }

        let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&point_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let point_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[point_light_count]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shadow_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_depth_views: [wgpu::TextureView; 6] = array_init::array_init(|i| 
            shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("shadow view"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: i as u32,
                array_layer_count: None,
            })
        );

        let mut shadow_buffers: Vec<wgpu::Buffer> = Vec::new();
        for perspective in g_perspectives {
            shadow_buffers.push(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("perspective buffer"),
                contents: cast_slice(&[Align16(perspective)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));
        }

        let mut shadow_bind_groups: Vec<wgpu::BindGroup> = Vec::new();
        for buffer in &shadow_buffers {
            shadow_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &Renderer::get_shadow_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding()
                    },
                ],
                label: Some("shadow bind group")
            }));
        }

        let mut directional_lights = Vec::new();

        let directional_light_components = world.read_component::<DirectionalLight>();
        let directional_light_count = directional_light_components.count() as i32 + 1;

        directional_lights.push(DirectionalData {
            _direction: Align16(cg::vec3(0.0, 0.0, 0.0)),
            _color: Align16([0.0, 0.0, 0.0])
        });

        for light in directional_light_components.join() {
            directional_lights.push(DirectionalData {
                _direction: Align16(light.direction),
                _color: Align16(light.color)
            });
        }

        let directional_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&directional_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let directional_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[directional_light_count]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let cube_view = shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("depthcube"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cubemap_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.1,
            lod_max_clamp: 15.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: Some(wgpu::SamplerBorderColor::OpaqueWhite),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_light_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: point_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: directional_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&cube_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&cubemap_sampler),
                },
            ],
            label: Some("light_bind_group"),
        });

        Self {
            point_lights,
            directional_lights,
            shadow: PointShadow { views: shadow_depth_views, cube_view, sampler: cubemap_sampler, buffers: shadow_buffers, bind_groups: shadow_bind_groups},
            bind_group,
            point_buffer,
            point_count_buffer,
            directional_buffer,
            directional_count_buffer,
        }
    }

    pub fn update_light_position(&mut self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        let index = index + 1;
        self.point_lights[index]._position = Align16(data);
        queue.write_buffer(&self.point_buffer, (std::mem::size_of::<LightData>() * index + std::mem::size_of::<Align16<[cg::Matrix4<f32>; 6]>>()) as u64, cast_slice(&[Align16(data)]));

        let proj = cg::perspective(cg::Deg(90.0), 1.0, 0.1, 15.0);
        let light_pos = data;
        let centers = vec![light_pos + cg::vec3(-1.0, 0.0, 0.0), light_pos + cg::vec3(1.0, 0.0, 0.0), light_pos + cg::vec3(0.0, -1.0, 0.0), light_pos + cg::vec3(0.0, 1.0, 0.0), light_pos + cg::vec3(0.0, 0.0, -1.0), light_pos + cg::vec3(0.0, 0.0, 1.0)];
        let up_vectors = vec![
            cg::vec3(0.0, -1.0, 0.0),  // Positive X
            cg::vec3(0.0, -1.0, 0.0),  // Negative X
            cg::vec3(0.0, 0.0, 1.0), // Positive Y
            cg::vec3(0.0, 0.0, -1.0), // Negative Y
            cg::vec3(0.0, -1.0, 0.0),  // Positive Z
            cg::vec3(0.0, -1.0, 0.0)  // Negative Z
        ];
        let perspectives = centers.iter().zip(up_vectors.iter()).map(|(center, up)| {
            proj * cg::Matrix4::look_at_rh(cg::point3(light_pos.x, light_pos.y, light_pos.z), cg::point3(center.x, center.y, center.z), *up)
        }).collect::<Vec<_>>();

        queue.write_buffer(&self.point_buffer, (std::mem::size_of::<LightData>() * index) as u64, cast_slice(&[Align16([perspectives[0], perspectives[1], perspectives[2], perspectives[3], perspectives[4], perspectives[5]])]));
        for (i, buffer) in self.shadow.buffers.iter().enumerate() {
            queue.write_buffer(&buffer, 0, cast_slice(&[Align16(perspectives[i])]));
        }
    }

    pub fn update_light_color(&self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        let index = index + 1;
        queue.write_buffer(&self.point_buffer, (std::mem::size_of::<LightData>() * index + std::mem::size_of::<Align16<[cg::Matrix4<f32>; 6]>>() + std::mem::size_of::<Align16<cg::Vector3<f32>>>()) as u64, cast_slice(&[data]));
    }

    pub fn update_directional_data(&self, queue: &wgpu::Queue, index: usize, direction: cg::Vector3<f32>, color: [f32; 3]) {
        let index = index + 1;
        queue.write_buffer(&self.directional_buffer, (std::mem::size_of::<DirectionalData>()  * index) as u64, cast_slice(&[DirectionalData {
            _direction: Align16(direction),
            _color: Align16(color)
        }]));
    }

    pub fn add_point_light(&mut self, device: &wgpu::Device, transform: &Transform, light: &PointLight) {
        let transform_data = transform.get_position();
        let light_data = light.get_color();

        self.point_lights.push(LightData {
            _position: Align16(transform_data),
            _color: Align16(light_data),
            _projection: Align16([cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity(), cg::SquareMatrix::identity()])
        });

        self.point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&self.point_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.point_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[self.point_lights.len() as i32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_light_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.point_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.directional_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });
    }

    pub fn remove_point_light(&mut self, device: &wgpu::Device, index: usize) {
        self.point_lights.remove(index + 1);

        self.point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&self.point_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.point_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[self.point_lights.len() as i32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_light_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.point_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.directional_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });
    }

    pub fn add_directional_light(&mut self, device: &wgpu::Device, light: &DirectionalLight) {
        self.directional_lights.push(DirectionalData {
            _direction: Align16(light.direction),
            _color: Align16(light.color)
        });

        self.directional_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&self.directional_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.directional_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[self.directional_lights.len() as i32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_light_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.point_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.directional_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });
    }

    pub fn remove_directional_light(&mut self, device: &wgpu::Device, index: usize) {
        self.directional_lights.remove(index + 1);

        self.directional_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&self.directional_lights.clone().into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.directional_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[self.directional_lights.len() as i32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Renderer::get_light_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.point_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.directional_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });
    }
}