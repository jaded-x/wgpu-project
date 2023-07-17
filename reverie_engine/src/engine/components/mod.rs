use super::registry::Registry;

pub mod transform;
pub mod mesh;
pub mod material;
pub mod name;
pub mod light;
pub mod parent;

pub trait ComponentDefault {
    fn default(device: &wgpu::Device, registry: &mut Registry) -> Self;
}

pub trait TypeName {
    fn type_name() -> &'static str;
}