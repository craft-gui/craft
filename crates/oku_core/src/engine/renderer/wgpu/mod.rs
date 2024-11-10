mod camera;
mod context;
mod pipeline_2d;
mod texture;
mod uniform;
mod vertex;

use crate::elements::element::ElementState;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, Renderer};
use crate::engine::renderer::wgpu::camera::Camera;
use crate::engine::renderer::wgpu::context::{
    create_surface_config, request_adapter, request_device_and_queue, Context,
};
use crate::engine::renderer::wgpu::pipeline_2d::Pipeline2D;
use crate::engine::renderer::wgpu::texture::Texture;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::components::component::{ComponentId, GenericUserState};
use cosmic_text::FontSystem;
use glyphon::{Cache, TextAtlas, TextRenderer, Viewport};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use wgpu::MultisampleState;
use winit::window::Window;

pub struct WgpuRenderer<'a> {
    context: Context<'a>,
    pipeline2d: Pipeline2D,
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(
        window: Arc<dyn Window>
    ) -> WgpuRenderer<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
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

        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_config.format);
        let text_renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        let context = Context {
            device,
            queue,
            default_texture,
            surface,
            surface_config,
            surface_clear_color: Color::new_from_rgba_u8(255, 255, 255, 255),
            is_srgba_format: false,

            glyphon_cache: cache,
            glyphon_viewport: viewport,
            glyphon_atlas: atlas,
            glyphon_text_renderer: text_renderer,
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

        self.pipeline2d.global_uniform.set_view_proj(&self.pipeline2d.camera);
        self.context.queue.write_buffer(
            &self.pipeline2d.global_buffer,
            0,
            bytemuck::cast_slice(&[self.pipeline2d.global_uniform.view_proj]),
        );
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.context.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.pipeline2d.draw_rect(rectangle, fill_color);
        // self.pipeline2d.draw_rect_outline(rectangle, Color::RED);
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        self.pipeline2d.draw_rect_outline(rectangle, outline_color);
    }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.pipeline2d.draw_text(element_id, rectangle, fill_color);
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.pipeline2d.draw_image(rectangle, resource_identifier)
    }

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &HashMap<ComponentId, Box<ElementState>>,
    ) {
        self.pipeline2d.submit(&mut self.context, resource_manager, font_system, element_state);
    }
}
