use crate::renderer::wgpu::camera::Camera;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    pub is_surface_srgb_format: u32,
    // wgpu requires that buffer bindings be padded to a multiple of 16 bytes, so we need to add 12 extra bytes here.
    pub _padding: [u32; 3],
    pub view_proj: [[f32; 4]; 4],
}

impl GlobalUniform {
    pub(crate) fn new() -> Self {
        Self {
            is_surface_srgb_format: 0,
            _padding: [0, 0, 0],
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(crate) fn set_is_surface_srgb_format(&mut self, is_surface_srgb_format: bool) {
        self.is_surface_srgb_format = is_surface_srgb_format as u32;
    }

    pub(crate) fn set_view_proj_with_camera(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }

    pub(crate) fn _set_view_proj(&mut self, glam: glam::Mat4) {
        self.view_proj = glam.to_cols_array_2d();
    }
}

pub struct GlobalBuffer {
    buffer: wgpu::Buffer,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl GlobalBuffer {
    pub fn new(device: &wgpu::Device, global_uniform: &GlobalUniform) -> Self {
        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::bytes_of(global_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("global_bind_group_layout"),
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_buffer.as_entire_binding(),
            }],
            label: Some("global_bind_group"),
        });

        Self {
            buffer: global_buffer,
            bind_group_layout: global_bind_group_layout,
            bind_group: global_bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, global_uniform: &GlobalUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*global_uniform]));
    }
}
