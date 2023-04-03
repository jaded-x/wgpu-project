use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub mesh_id: usize,
}

impl Mesh {
    pub fn new(mesh_id: usize) -> Self {
        Self {
            mesh_id
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vert {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
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
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}