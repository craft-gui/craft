use crate::components::component::ComponentId;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::engine::renderer::wgpu::texture::Texture;
use crate::engine::renderer::wgpu::vertex::Vertex;
use crate::platform::resource_manager::{ResourceIdentifier};

fn bind_group_from_2d_texture(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    texture: &Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: Some("oku_bind_group"),
    })
}

pub struct RectangleBatch {
    texture_path: Option<ResourceIdentifier>,
    rectangle_vertices: Vec<Vertex>,
    rectangle_indices: Vec<u32>,
}

pub struct TextRenderInfo {
    pub(crate) element_id: ComponentId,
    pub(crate) rectangle: Rectangle,
    pub(crate) fill_color: Color,
}
