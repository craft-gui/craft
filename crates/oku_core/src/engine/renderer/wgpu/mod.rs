mod camera;
mod context;
mod pipeline_2d;
mod texture;
mod uniform;
mod vertex;
mod rectangle;
mod text;
mod render_group;

use crate::components::component::ComponentId;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand, Renderer};
use crate::engine::renderer::wgpu::camera::Camera;
use crate::engine::renderer::wgpu::context::{
    create_surface_config, request_adapter, request_device_and_queue, Context,
};
use crate::engine::renderer::wgpu::texture::Texture;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::state_store::StateStore;
use cosmic_text::FontSystem;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use winit::window::Window;
use crate::engine::renderer::wgpu::rectangle::pipeline::DEFAULT_PIPELINE_CONFIG;
use crate::engine::renderer::wgpu::rectangle::RectangleRenderer;
use crate::engine::renderer::wgpu::render_group::{ClipRectangle, RenderGroup};
use crate::engine::renderer::wgpu::text::text::TextRenderer;

pub struct WgpuRenderer<'a> {
    context: Context<'a>,
    // pipeline2d: Pipeline2D,
    rectangle_renderer: RectangleRenderer,
    text_renderer: TextRenderer,
    render_commands: Vec<RenderCommand>
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(window: Arc<dyn Window>) -> WgpuRenderer<'a> {
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

        let context = Context {
            camera: Camera {
                width: surface_config.width as f32,
                height: surface_config.height as f32,
                z_near: 0.0,
                z_far: 100.0,
            },
            device,
            queue,
            default_texture,
            surface,
            surface_config,
            surface_clear_color: Color::rgba(255, 255, 255, 255),
            is_srgba_format: false,
        };

        // let pipeline2d = Pipeline2D::new(&context);
        let rectangle_renderer = RectangleRenderer::new(&context);
        let text_renderer = TextRenderer::new(&context);

        WgpuRenderer {
            // pipeline2d,
            context,
            rectangle_renderer,
            text_renderer,
            render_commands: vec![],
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
        self.context.camera = Camera {
            width,
            height,
            z_near: 0.0,
            z_far: 100.0,
        };

        // self.pipeline2d.global_uniform.set_view_proj_with_camera(&self.context.camera);
        
        let rect_pipeline = self.rectangle_renderer.cached_pipelines.get_mut(&DEFAULT_PIPELINE_CONFIG).unwrap();
        rect_pipeline.global_uniform.set_view_proj_with_camera(&self.context.camera);
        
        // self.context.queue.write_buffer(
        //     &self.pipeline2d.global_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.pipeline2d.global_uniform.view_proj]),
        // );
        self.context.queue.write_buffer(
            &rect_pipeline.global_buffer,
            0,
            bytemuck::cast_slice(&[rect_pipeline.global_uniform.view_proj]),
        );
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.context.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        //self.pipeline2d.draw_rect_outline(rectangle, outline_color);
    }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        //self.pipeline2d.draw_image(rectangle, resource_identifier)
    }

    fn push_layer(&mut self, clip_rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(clip_rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &StateStore,
    ) {

        let render_commands_len = self.render_commands.len();
       
        if render_commands_len == 0 {
            return;
        }
        
        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = self.context.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let r = self.context.surface_clear_color.r / 255.0;
        let g = self.context.surface_clear_color.g / 255.0;
        let b = self.context.surface_clear_color.b / 255.0;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: self.context.surface_clear_color.a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let render_commands = self.render_commands.drain(..);
            
            let viewport_clip_rect = Rectangle {
                x: 0.0,
                y: 0.0,
                width: self.context.surface_config.width as f32,
                height: self.context.surface_config.height as f32
            };
            
            let mut render_groups: Vec<RenderGroup> = Vec::new();
            render_groups.push(RenderGroup {
                clip_rectangle: viewport_clip_rect,
            });

            for (index, command) in render_commands.enumerate() {
                let mut should_submit = index == render_commands_len - 1;

                match command {
                    RenderCommand::PushLayer(clip_rectangle) => {
                        let parent_clip_rectangle = render_groups.last().unwrap().clip_rectangle;
                        let constrained_clip_rectangle = clip_rectangle.constrain_to_clip_rectangle(&parent_clip_rectangle);
                        render_groups.push(RenderGroup {
                            clip_rectangle: constrained_clip_rectangle
                        });
                        
                        render_pass.set_scissor_rect(
                            parent_clip_rectangle.x as u32,
                            parent_clip_rectangle.y as u32,
                            parent_clip_rectangle.width as u32,
                            parent_clip_rectangle.height as u32,
                        );

                        let rectangle_renderer_per_frame_data = self.rectangle_renderer.prepare(&self.context);
                        self.text_renderer.prepare(&self.context, font_system, element_state, parent_clip_rectangle);
                       
                        self.rectangle_renderer.draw(&mut render_pass, rectangle_renderer_per_frame_data);
                        self.text_renderer.draw(&mut render_pass);
                    }
                    RenderCommand::PopLayer => {
                        if should_submit {
                            should_submit = false;
                        }
                        let current_clip_rectangle = render_groups.pop().unwrap().clip_rectangle;

                        render_pass.set_scissor_rect(
                            current_clip_rectangle.x as u32,
                            current_clip_rectangle.y as u32,
                            current_clip_rectangle.width as u32,
                            current_clip_rectangle.height as u32,
                        );

                        let rectangle_renderer_per_frame_data = self.rectangle_renderer.prepare(&self.context);
                        self.text_renderer.prepare(&self.context, font_system, element_state, current_clip_rectangle);
                        
                        self.rectangle_renderer.draw(&mut render_pass, rectangle_renderer_per_frame_data);
                        self.text_renderer.draw(&mut render_pass);
                    }
                    RenderCommand::DrawRect(rectangle, fill_color) => {
                        self.rectangle_renderer.build(rectangle, fill_color);
                    }
                    RenderCommand::DrawRectOutline(_, _) => {}
                    RenderCommand::DrawImage(_, _) => {}
                    RenderCommand::DrawText(rectangle, component_id, color) => {
                        self.text_renderer.build(rectangle, component_id, color);
                        println!("{:?} {:?}", rectangle, component_id);
                    }
                }

                if should_submit {
                    let current_clip_rectangle = render_groups.pop().unwrap().clip_rectangle;

                    render_pass.set_scissor_rect(
                        current_clip_rectangle.x as u32,
                        current_clip_rectangle.y as u32,
                        current_clip_rectangle.width as u32,
                        current_clip_rectangle.height as u32,
                    );

                    let rectangle_renderer_per_frame_data = self.rectangle_renderer.prepare(&self.context);
                    self.text_renderer.prepare(&self.context, font_system, element_state, current_clip_rectangle);
                    
                    self.rectangle_renderer.draw(&mut render_pass, rectangle_renderer_per_frame_data);
                    self.text_renderer.draw(&mut render_pass);
                }

            }
        }
        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
    }
}
