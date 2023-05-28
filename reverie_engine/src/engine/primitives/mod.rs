pub mod cube;

use super::model::Mesh;

pub struct PrimitiveMesh {
    pub mesh: Mesh,
}

pub enum Primitive {
    Cube,
    Plane,
}