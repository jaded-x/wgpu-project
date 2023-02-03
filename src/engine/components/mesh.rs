use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vert {
    pub position: [f32; 3],
}

impl Vert {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}