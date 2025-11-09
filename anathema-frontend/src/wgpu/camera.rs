use anathema_geometry::{Pos, Size};
use glam::{Mat4, Vec3};

pub struct Camera {
    projection: Mat4,
    pub position: Pos,
    pub rotation: f32,
    pub scale: Size,
}

impl Camera {
    pub fn new(size: Size, near: f32, far: f32) -> Self {
        Self {
            projection: Self::projection_from_size(size, near, far),
            position: Pos::ZERO,
            rotation: 0.0,
            scale: Size::new(1.0, 1.0),
        }
    }

    fn projection_from_size(size: Size, near: f32, far: f32) -> Mat4 {
        let right = size.width / 2.0;
        let left = -right;
        let top = size.height / 2.0;
        let bottom = -top;

        Mat4::orthographic_lh(left, right, bottom, top, near, far)
    }

    /// Create the projection matrix
    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(-Vec3::from((*self.position, 0.0)));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::from((self.scale.to_vec(), 1.0)));
        let view = scale * rotation * translation;
        self.projection * view
    }

    pub fn translate(&mut self, new_pos: Pos) {
        self.position = new_pos;
    }

    pub fn translate_by(&mut self, new_pos: Pos) {
        *self.position += *new_pos;
    }

    pub(crate) fn resize(&mut self, size: Size, near: f32, far: f32) {
        self.projection = Self::projection_from_size(size, near, far);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wgpu::ctx::{FAR, NEAR};

    #[test]
    fn camera_output() {
        let width = 800.0;
        let height = 600.0;
        let mut camera = Camera::new(Size::new(width, height), NEAR, FAR);
        panic!("{:#?}", camera.projection);
    }
}
