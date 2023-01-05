use crate::util::align::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub color: Align16<[f32; 3]>,
}