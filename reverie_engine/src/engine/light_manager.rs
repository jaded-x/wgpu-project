use specs::{*, WorldExt};
use wgpu::util::DeviceExt;

use crate::util::{cast_slice, align::Align16};

use super::components::{light::{PointLight, DirectionalLight}, transform::Transform};

#[derive(Clone)]
struct LightData {
    _position: Align16<cg::Vector3<f32>>,
    _color: Align16<cg::Vector3<f32>>
}

#[derive(Clone)]
struct DirectionalData {
    _direction: Align16<cg::Vector3<f32>>,
    _color: Align16<[f32; 3]>
}

pub struct LightManager {
    pub bind_group: wgpu::BindGroup,
    pub point_buffer: wgpu::Buffer,
    pub point_count_buffer: wgpu::Buffer,
    pub directional_buffer: wgpu::Buffer,
    pub directional_count_buffer: wgpu::Buffer,
}

impl LightManager {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, world: &World) -> Self {
        let mut point_lights = Vec::new();

        let transform_components = world.read_component::<Transform>();
        let point_light_components = world.read_component::<PointLight>();

        let point_light_count = point_light_components.count() as i32 + 1;

        point_lights.push(LightData {
            _position: Align16(cg::vec3(0.0, 0.0, 0.0)),
            _color: Align16(cg::vec3(0.0, 0.0, 0.0)),
        });

        for (transform, light) in (&transform_components, &point_light_components).join() {
            let transform_data = transform.get_position();
            let light_data = light.get_color();

            point_lights.push(LightData {
                _position: Align16(transform_data),
                _color: Align16(light_data)
            });
        }

        let point_lights_data = point_lights.clone();

        let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_data_buffer"),
            contents: cast_slice(&point_lights_data.into_boxed_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let point_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_count_buffer"),
            contents: cast_slice(&[point_light_count]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
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
            ],
            label: Some("light_bind_group"),
        });

        Self {
            bind_group,
            point_buffer,
            point_count_buffer,
            directional_buffer,
            directional_count_buffer,
        }
    }

    pub fn update_light_position(&self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        queue.write_buffer(&self.point_buffer, (std::mem::size_of::<LightData>() * index) as u64, cast_slice(&[data]));
    }

    pub fn update_light_data(&self, queue: &wgpu::Queue, index: usize, data: cg::Vector3<f32>) {
        queue.write_buffer(&self.point_buffer, (std::mem::size_of::<LightData>()  * index  + 16) as u64, cast_slice(&[data]));
    }

    pub fn update_directional_data(&self, queue: &wgpu::Queue, index: usize, direction: cg::Vector3<f32>, color: [f32; 3]) {
        queue.write_buffer(&self.directional_buffer, (std::mem::size_of::<DirectionalData>()  * index) as u64, cast_slice(&[DirectionalData {
            _direction: Align16(direction),
            _color: Align16(color)
        }]));
    }
}