use specs::{prelude::*, Component};

#[derive(Component)]
#[storage(VecStorage)]
pub struct PointLight {
    diffuse_color: [f32; 3],
}

impl PointLight {
    pub fn new(diffuse_color: [f32; 3]) -> Self {
        Self {
            diffuse_color,
        }
    }

    pub fn get_color(&self) -> cg::Vector3<f32> {
        cg::vec3(self.diffuse_color[0], self.diffuse_color[1], self.diffuse_color[2])
    }
}
