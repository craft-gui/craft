mod camera;
mod context;
mod globals;
mod text;
mod render_group;
mod image;
pub(crate) mod texture;
mod path;

use crate::components::component::ComponentId;
use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, Renderer};
use crate::renderer::wgpu::camera::Camera;
use crate::renderer::wgpu::context::{create_surface_config, request_adapter, request_device_and_queue, Context};
use crate::renderer::wgpu::globals::{GlobalBuffer, GlobalUniform};
use crate::renderer::wgpu::image::image::{ImagePerFrameData, ImageRenderer};
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo::BezPath;
use std::sync::Arc;
use lyon::geom::{point, Box2D};
use lyon::path::{Path, Winding};
use peniko::color::palette;
use tokio::sync::RwLockReadGuard;
use vello::kurbo;
use winit::window::Window;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::wgpu::path::PathRenderer;
use crate::renderer::wgpu::render_group::{ClipRectangle, RenderGroup};
use crate::renderer::wgpu::text::text::TextRenderer;
use crate::renderer::wgpu::texture::Texture;

pub struct WgpuRenderer<'a> {
    context: Context<'a>,
    text_renderer: TextRenderer,
    image_renderer: ImageRenderer,
    path_renderer: PathRenderer,
    render_commands: Vec<RenderCommand>,
    render_snapshots: Vec<RenderSnapshot>,
}

pub struct PerFrameData {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) indices: usize,
}

pub struct RenderSnapshot {
    text_per_frame_data: Option<PerFrameData>,
    image_per_frame_data: Option<ImagePerFrameData>,
    path_per_frame_data: Option<PerFrameData>,
    clip_rectangle: Rectangle,
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

        let camera = Camera {
            width: surface_config.width as f32,
            height: surface_config.height as f32,
            z_near: 0.0,
            z_far: 100.0,
        };
        let mut global_buffer_uniform = GlobalUniform::new();
        global_buffer_uniform.set_view_proj_with_camera(&camera);

        let global_buffer = GlobalBuffer::new(&device, &global_buffer_uniform);

        let default_texture = Texture::generate_default_white_texture(&device, &queue);
        
        let context = Context {
            camera,
            device,
            queue,
            global_buffer,
            global_buffer_uniform,
            surface,
            surface_config,
            surface_clear_color: Color::WHITE,
            is_srgba_format: false,
            default_texture
        };
        let text_renderer = TextRenderer::new(&context);
        let image_renderer = ImageRenderer::new(&context);
        let path_renderer = PathRenderer::new(&context);

        WgpuRenderer {
            context,
            text_renderer,
            image_renderer,
            path_renderer,
            render_commands: vec![],
            render_snapshots: vec![],
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


        self.context.global_buffer_uniform.set_view_proj_with_camera(&self.context.camera);
        self.context.global_buffer.update(&self.context.queue, &self.context.global_buffer_uniform);
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

    fn fill_bez_path(&mut self, path: BezPath, color: Color) {
        self.render_commands.push(RenderCommand::FillBezPath(path, color));
    }

    fn fill_lyon_path(&mut self, path: &Path, color: Color) {
    }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, clip_rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(clip_rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn prepare(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>, font_system: &mut FontSystem, element_state: &ElementStateStore) {

        let mut collect_render_snapshots = |render_commands: &mut Vec<RenderCommand>| {
            let render_commands_len = render_commands.len();
            
            if render_commands_len == 0 {
                return;
            }
            
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
            
            for (index, command) in render_commands.drain(..).enumerate() {
                let mut should_submit = index == render_commands_len - 1;

                match command {
                    RenderCommand::PushLayer(clip_rectangle) => {
                        let parent_clip_rectangle = render_groups.last().unwrap().clip_rectangle;
                        let constrained_clip_rectangle = clip_rectangle.constrain_to_clip_rectangle(&parent_clip_rectangle);
                        render_groups.push(RenderGroup {
                            clip_rectangle: constrained_clip_rectangle
                        });

                        let snapshot = assemble_render_snapshot(&mut self.context, font_system, element_state, &mut self.text_renderer, &mut self.image_renderer, &mut self.path_renderer, parent_clip_rectangle);
                        self.render_snapshots.push(snapshot);
                    }
                    RenderCommand::PopLayer => {
                        should_submit = true;
                    }
                    RenderCommand::DrawRect(rectangle, fill_color) => {
                        self.path_renderer.build_rectangle(rectangle, fill_color);
                    }
                    RenderCommand::DrawRectOutline(_, _) => {}
                    RenderCommand::DrawImage(rectangle, resource_identifier) => {
                        self.image_renderer.build(rectangle, resource_identifier.clone(), Color::WHITE);
                    }
                    RenderCommand::DrawText(rectangle, component_id, color) => {
                        self.text_renderer.build(rectangle, component_id, color);
                    }
                    RenderCommand::FillBezPath(bez_path, color) => {
                        let mut builder = lyon::path::Path::builder();
                        for element in bez_path.iter() {
                            match element {
                                kurbo::PathEl::MoveTo(p) => {
                                    builder.begin(lyon::geom::euclid::Point2D::new(p.x as f32, p.y as f32));
                                }
                                kurbo::PathEl::LineTo(p) => {
                                    builder.line_to(lyon::geom::euclid::Point2D::new(p.x as f32, p.y as f32));
                                }
                                kurbo::PathEl::QuadTo(ctrl, to) => {
                                    builder.quadratic_bezier_to(
                                        lyon::geom::euclid::Point2D::new(ctrl.x as f32, ctrl.y as f32),
                                        lyon::geom::euclid::Point2D::new(to.x as f32, to.y as f32),
                                    );
                                }
                                kurbo::PathEl::CurveTo(ctrl1, ctrl2, to) => {
                                    builder.cubic_bezier_to(
                                        lyon::geom::euclid::Point2D::new(ctrl1.x as f32, ctrl1.y as f32),
                                        lyon::geom::euclid::Point2D::new(ctrl2.x as f32, ctrl2.y as f32),
                                        lyon::geom::euclid::Point2D::new(to.x as f32, to.y as f32),
                                    );
                                }
                                kurbo::PathEl::ClosePath => {
                                    builder.end(true);
                                }
                            }
                        }

                        let path = builder.build();
                        self.path_renderer.build(path, color);
                    },
                    RenderCommand::FillLyonPath(path, color) => {
                        self.path_renderer.build(path, color);
                    }
                }

                if should_submit {
                    let current_clip_rectangle = render_groups.pop().unwrap().clip_rectangle;
                    let snapshot = assemble_render_snapshot(&mut self.context, font_system, element_state, &mut self.text_renderer, &mut self.image_renderer, &mut self.path_renderer, current_clip_rectangle);
                    self.render_snapshots.push(snapshot);
                }
            };
        };
      
        collect_render_snapshots(&mut self.render_commands);
    }
    


    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>) {
        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = self.context.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let [r, g, b, a] = self.context.surface_clear_color.components;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Oku Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            for snapshot in &self.render_snapshots {
                
                let clip_rectangle = ClipRectangle::constrain_to_clip_rectangle(&snapshot.clip_rectangle, &ClipRectangle {
                    x: 0.0,
                    y: 0.0,
                    width: output.texture.width() as f32,
                    height: output.texture.height() as f32,
                });
                render_pass.set_scissor_rect(
                    clip_rectangle.x as u32,
                    clip_rectangle.y as u32,
                    clip_rectangle.width as u32,
                    clip_rectangle.height as u32,
                );

                if let Some(path_per_frame_data) = snapshot.path_per_frame_data.as_ref() {
                    self.path_renderer.draw(&mut self.context, &mut render_pass, path_per_frame_data);
                }
                
                if let Some(text_per_frame_data) = snapshot.text_per_frame_data.as_ref() {
                    self.text_renderer.draw(&mut self.context, &mut render_pass, text_per_frame_data);
                }

                if let Some(image_per_frame_data) = snapshot.image_per_frame_data.as_ref() {
                    self.image_renderer.draw(&mut self.context, &resource_manager, &mut render_pass, image_per_frame_data);
                }
                
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.render_snapshots.clear();
    }
}

fn assemble_render_snapshot(
        context: &mut Context,
        font_system: &mut FontSystem,
        element_state: &ElementStateStore,
        text_renderer: &mut TextRenderer,
        image_renderer: &mut ImageRenderer,
        path_renderer: &mut PathRenderer,
        clip_rectangle: ClipRectangle
) -> RenderSnapshot {
    let text_renderer_per_frame_data = text_renderer.prepare(context, font_system, element_state);
    let image_renderer_per_frame_data = image_renderer.prepare(context);
    let path_renderer_per_frame_data = path_renderer.prepare(context);

    RenderSnapshot {
        text_per_frame_data: text_renderer_per_frame_data,
        image_per_frame_data: image_renderer_per_frame_data,
        path_per_frame_data: path_renderer_per_frame_data,
        clip_rectangle,
    }
}
