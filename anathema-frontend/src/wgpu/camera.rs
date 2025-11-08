use anathema_geometry::{Mat4x4, Pos, Size};

pub struct Camera {
    projection: Mat4x4,
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

    fn projection_from_size(size: Size, near: f32, far: f32) -> Mat4x4 {
        let right = size.width / 2.0;
        let left = -right;
        let top = size.height / 2.0;
        let bottom = -top;

        Mat4x4::ortho(left, right, bottom, top, near, far)
    }

    /// Create the projection matrix
    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        let translation = Mat4x4::from_translation((-self.position).to_vec());
        let rotation = Mat4x4::from_rotation(self.rotation);
        let scale = Mat4x4::from_scale(self.scale);
        let view = scale * rotation * translation;
        (self.projection * view).into()
    }

    pub fn translate(&mut self, new_pos: Pos) {
        self.position = new_pos;
    }

    pub fn translate_by(&mut self, new_pos: Pos) {
        self.position += new_pos;
    }

    pub(crate) fn resize(&mut self, size: Size, near: f32, far: f32) {
        self.projection = Self::projection_from_size(size, near, far);
    }
}
