#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleVertex {
    pub position: [f32; 3],             // 12 bytes
    pub size: [f32; 2],                 // 8 bytes
    pub background_color: [f32; 4],     // 16 bytes (assuming Color is [f32; 4])
    pub border_color: [[f32; 4]; 4],    // 64 bytes (4 * 16 bytes)
    pub border_radius: [f32; 4],        // 16 bytes
    pub border_thickness: [f32; 4],     // 16 bytes
}

// FIXME: Make a builder for this.
impl RectangleVertex {
    pub(crate) fn description<'a>() -> wgpu::VertexBufferLayout<'a> {

        wgpu::VertexBufferLayout {
            array_stride: size_of::<RectangleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position - Float32x3
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // size - Float32x2
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // background color - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>() + size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border color top - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border color right - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border color bottom - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border color left - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border radius - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>() * 5) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border thickness - Float32x4
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>()
                        + size_of::<[f32; 2]>()
                        + size_of::<[f32; 4]>() * 6) as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
