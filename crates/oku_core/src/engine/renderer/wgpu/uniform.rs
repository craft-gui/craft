use crate::engine::renderer::wgpu::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    pub is_srgb_format: u32,
    // wgpu requires that buffer bindings be padded to a multiple of 16 bytes, so we need to add 12 extra bytes here.
    pub _padding: [u32; 3],
    pub view_proj: [[f32; 4]; 4],
}

impl GlobalUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            is_srgb_format: 0,
            _padding: [0, 0, 0],
        }
    }
    
    pub(crate) fn set_is_srgb_format(&mut self, is_srgb_format: bool) {
        self.is_srgb_format = is_srgb_format as u32;
    }

    pub(crate) fn set_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}
