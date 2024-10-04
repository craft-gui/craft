mod texture;
mod context;
mod uniform;
mod pipeline_2d;
mod vertex;
mod camera;

use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, Renderer};
use glam;
use image::{GenericImageView, ImageEncoder};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::engine::renderer::wgpu::camera::Camera;
use crate::engine::renderer::wgpu::context::{create_surface_config, request_adapter, request_device_and_queue, Context};
use crate::engine::renderer::wgpu::pipeline_2d::Pipeline2D;
use crate::engine::renderer::wgpu::texture::Texture;

pub struct WgpuRenderer<'a> {
    context: Context<'a>,
    pipeline2d: Pipeline2D,
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(window: Arc<dyn Window>) -> WgpuRenderer<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12 | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = request_adapter(instance, &surface).await;
        let (device, queue) = request_device_and_queue(&adapter).await;

        let surface_size = window.surface_size();
        let surface_config =
            create_surface_config(&surface, surface_size.width, surface_size.height, &device, &adapter);
        surface.configure(&device, &surface_config);

        let default_texture = Texture::generate_default_white_texture(&device, &queue);

        let context = Context {
            device,
            queue,
            default_texture,
            surface,
            surface_config,
            surface_clear_color: Color::new_from_rgba_u8(255, 255, 255, 255),
        };

        let pipeline2d = Pipeline2D::new(&context);
        
        WgpuRenderer {
            pipeline2d,
            context,
        }
    }
}

impl Renderer for WgpuRenderer<'_> {
    fn surface_width(&self) -> f32 {
        self.context.surface_config.width as f32
    }

    fn surface_height(&self) -> f32 {
        self.context.surface_config.height as f32
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.context.surface_config.width = width as u32;
        self.context.surface_config.height = height as u32;
        self.context.surface.configure(&self.context.device, &self.context.surface_config);
        self.pipeline2d.camera = Camera {
            width,
            height,
            z_near: 0.0,
            z_far: 100.0,
        };

        self.pipeline2d.camera_uniform.update_view_proj(&self.pipeline2d.camera);
        self.context.queue.write_buffer(&self.pipeline2d.camera_buffer, 0, bytemuck::cast_slice(&[self.pipeline2d.camera_uniform.view_proj]));
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.context.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
       self.pipeline2d.draw_rect(rectangle, fill_color);
    }

    fn draw_image(&mut self, rectangle: Rectangle, path: &str) {
        self.pipeline2d.draw_image(rectangle, path)
    }

    fn submit(&mut self) {
        self.pipeline2d.submit(&mut self.context)
    }     
}
