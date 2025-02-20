use crate::renderer::color::Color;
use crate::renderer::wgpu::camera::Camera;
use crate::renderer::wgpu::globals::{GlobalBuffer, GlobalUniform};
use crate::renderer::wgpu::texture::Texture;
use wgpu::{CompositeAlphaMode, PresentMode};
use oku_logging::info;

pub struct Context<'a> {
    pub(crate) camera: Camera,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) global_buffer: GlobalBuffer,
    pub(crate) global_buffer_uniform: GlobalUniform,
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) surface_clear_color: Color,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) is_surface_srgba_format: bool,
    pub(crate) default_texture: Texture,
}

pub async fn request_adapter(instance: wgpu::Instance, surface: &wgpu::Surface<'_>) -> wgpu::Adapter {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request an adapter, cannot request GPU access without an adapter.");
    adapter
}

pub async fn request_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: wgpu::Label::from("oku_wgpu_renderer"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None, // Trace path
        )
        .await
        .expect("Failed to request a GPU!");
    (device, queue)
}

pub fn create_surface_config(
    surface: &wgpu::Surface<'_>,
    width: u32,
    height: u32,
    _device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);

    // Try to find any available sRGB format first:
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        // If no sRGB formats are available, then fall back to Rgba8Unorm.
        .unwrap_or_else(|| {
            if surface_caps.formats.contains(&wgpu::TextureFormat::Rgba8Unorm) {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                // Guaranteed to be available in Wgpu.
                wgpu::TextureFormat::Bgra8Unorm
            }
        });
    
    info!("Surface format: {:?}", surface_format);
    
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 0,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    }
}
