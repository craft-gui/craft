#[derive(Debug)]
pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub(crate) fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::IDENTITY;
        let proj = glam::Mat4::orthographic_lh(0.0, self.width, self.height, 0.0, self.z_near, self.z_far);
        proj * view
    }
}
