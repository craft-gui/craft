mod render_context;
mod tinyvg;

use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::image_adapter::ImageAdapter;
use crate::renderer::renderer::{RenderCommand, Renderer as CraftRenderer, TextScroll};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::{FontSystem};
use peniko::kurbo::BezPath;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use vello_common::glyph::Glyph;
use vello_common::kurbo::{Affine, Rect};
use vello_common::paint::Paint;
use vello_common::peniko::Blob;
use vello_common::{kurbo, peniko};
use vello_hybrid::RenderSize;
use vello_hybrid::{RenderTargetConfig, Renderer};
use wgpu::RenderPassDescriptor;
use wgpu::TextureFormat;
use winit::window::Window;

use crate::renderer::text::BufferGlyphs;
use crate::renderer::vello_hybrid::render_context::RenderContext;
use crate::renderer::vello_hybrid::render_context::RenderSurface;
use crate::renderer::vello_hybrid::tinyvg::draw_tiny_vg;
use crate::renderer::Brush;
use vello_hybrid::Scene;

pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<dyn Window>,
}

enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended,
}

pub struct VelloHybridRenderer<'a> {
    render_commands: Vec<RenderCommand>,

    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState<'a>,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        &RenderTargetConfig {
            format: surface.config.format,
            width: surface.config.width,
            height: surface.config.height,
        },
    )
}

impl<'a> VelloHybridRenderer<'a> {
    pub(crate) async fn new(window: Arc<dyn Window>) -> VelloHybridRenderer<'a> {
        // Create a vello Surface
        let surface_size = window.surface_size();

        let mut vello_renderer = VelloHybridRenderer {
            render_commands: vec![],
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended,
            scene: Scene::new(surface_size.width as u16, surface_size.height as u16),
            surface_clear_color: Color::WHITE,
        };

        let surface = vello_renderer
            .context
            .create_surface(
                window.clone(),
                surface_size.width,
                surface_size.height,
                wgpu::PresentMode::AutoVsync,
                TextureFormat::Bgra8Unorm,
            )
            .await;

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState { window, surface });

        vello_renderer
    }

    fn prepare_with_render_commands(
        scene: &mut Scene,
        resource_manager: &Arc<ResourceManager>,
        font_system: &mut FontSystem,
        render_commands: &mut Vec<RenderCommand>,
    ) {
        for command in render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(_rectangle, _outline_color) => {
                    // vello_draw_rect_outline(&mut self.scene, rectangle, outline_color);
                }
                RenderCommand::DrawImage(_rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(resource) = resource {
                        if let Resource::Image(resource) = resource.as_ref() {
                            let image = &resource.image;
                            let data = Arc::new(ImageAdapter::new(resource.clone()));
                            let blob = Blob::new(data);
                            let _vello_image =
                                peniko::Image::new(blob, peniko::ImageFormat::Rgba8, image.width(), image.height());

                            /*   let mut transform = Affine::IDENTITY;
                               transform =
                                   transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                               transform = transform.pre_scale_non_uniform(
                                   rectangle.width as f64 / image.width() as f64,
                                   rectangle.height as f64 / image.height() as f64,
                               );*/

                            //scene.draw_image(&vello_image, transform);
                        }   
                    }
                }
                RenderCommand::DrawText(buffer_glyphs, rect, text_scroll, show_cursor) => {
                    let text_transform = Affine::translate((rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    // Draw the Glyphs
                    for buffer_line in &buffer_glyphs.buffer_lines {
                        for glyph_highlight in &buffer_line.glyph_highlights {
                            scene.set_paint(Paint::Solid(buffer_glyphs.glyph_highlight_color.premultiply().to_rgba8()));
                            scene.set_transform(text_transform);
                            scene.fill_rect(glyph_highlight);
                        }

                        if show_cursor {
                            if let Some(cursor) = &buffer_line.cursor {
                                scene.set_paint(Paint::Solid(buffer_glyphs.cursor_color.premultiply().to_rgba8()));
                                scene.set_transform(text_transform);
                                scene.fill_rect(cursor);
                            }
                        }

                        for glyph_run in &buffer_line.glyph_runs {
                            let font = font_system.get_font(glyph_run.font).unwrap().as_peniko();
                            let glyph_color = glyph_run.glyph_color;
                            let glyphs = glyph_run.glyphs.clone();
                            scene.set_paint(Paint::Solid(glyph_color.premultiply().to_rgba8()));
                            scene.reset_transform();
                            let glyph_run_builder = scene
                                .glyph_run(&font)
                                .font_size(buffer_glyphs.font_size)
                                .glyph_transform(text_transform);
                            glyph_run_builder.fill_glyphs(glyphs.iter().map(|glyph| Glyph {
                                id: glyph.glyph_id as u32,
                                x: glyph.x,
                                y: glyph.y + glyph_run.line_y,
                            }))
                        }
                    }
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier) => {
                    draw_tiny_vg(scene, rectangle, resource_manager, resource_identifier);
                }
                RenderCommand::PushLayer(rect) => {
                    let _clip = Rect::new(
                        rect.x as f64,
                        rect.y as f64,
                        (rect.x + rect.width) as f64,
                        (rect.y + rect.height) as f64,
                    );
                    //scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                }
                RenderCommand::PopLayer => {
                    //scene.pop_layer();
                }
                RenderCommand::FillBezPath(path, brush) => {
                    scene.set_paint(brush_to_paint(&brush));
                    scene.fill_path(&path);
                }
            }
        }
    }
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {
    scene.set_paint(Paint::Solid(fill_color.premultiply().to_rgba8()));
    scene.fill_rect(&rectangle.to_kurbo());
}

impl CraftRenderer for VelloHybridRenderer<'_> {
    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window.surface_size().width as f32,
            RenderState::Suspended => 0.0,
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window.surface_size().height as f32,
            RenderState::Suspended => 0.0,
        }
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };

        self.context.resize_surface(&mut render_state.surface, width as u32, height as u32);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, _rectangle: Rectangle, _outline_color: Color) {}

    fn fill_bez_path(&mut self, path: BezPath, brush: Brush) {
        self.render_commands.push(RenderCommand::FillBezPath(path, brush));
    }

    fn draw_text(
        &mut self,
        buffer_glyphs: BufferGlyphs,
        rectangle: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        self.render_commands.push(RenderCommand::DrawText(buffer_glyphs, rectangle, text_scroll, show_cursor));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn draw_tiny_vg(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawTinyVg(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn prepare(&mut self, resource_manager: Arc<ResourceManager>, _font_system: &mut FontSystem) {
        VelloHybridRenderer::prepare_with_render_commands(
            &mut self.scene,
            &resource_manager,
            _font_system,
            &mut self.render_commands,
        );
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };

        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let _width = surface.config.width;
        let _height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface.surface.get_current_texture().expect("failed to get surface texture");

        let render_size = RenderSize {
            width: surface.config.width,
            height: surface.config.height,
        };

        self.renderers[surface.dev_id].as_mut().unwrap().prepare(
            &device_handle.device,
            &device_handle.queue,
            &self.scene,
            &render_size,
        );

        let mut encoder = device_handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Vello Render to Surface pass"),
        });

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let clear_color = self.surface_clear_color.to_rgba8();
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render to Texture Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64 / 255.0,
                            g: clear_color.g as f64 / 255.0,
                            b: clear_color.b as f64 / 255.0,
                            a: clear_color.a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render to the surface's texture
            self.renderers[surface.dev_id].as_mut().unwrap().render(&self.scene, &mut pass);
        }

        device_handle.queue.submit([encoder.finish()]);

        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}

fn brush_to_paint(brush: &Brush) -> Paint {
    match brush {
        Brush::Color(color) => {
            Paint::Solid(color.premultiply().to_rgba8())
        }
        Brush::Gradient(gradient) => {
            // Paint::Gradient does not exist yet, so we need to come back and fix this later.
            let color = gradient.stops.first().map(|c| c.color.to_alpha_color()).unwrap_or(Color::BLACK);
            Paint::Solid(color.premultiply().to_rgba8())
        }
    }
}