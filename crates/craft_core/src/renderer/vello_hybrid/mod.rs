mod render_context;
mod tinyvg;

use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::image_adapter::ImageAdapter;
use crate::renderer::renderer::{RenderCommand, RenderList, Renderer as CraftRenderer, SortedCommands, TextScroll};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::ResourceManager;
use std::sync::Arc;
use peniko::kurbo::Shape;
use vello_common::glyph::Glyph;
use vello_common::paint::{Paint};
use vello_common::peniko::Blob;
use vello_common::{kurbo, peniko};
use vello_hybrid::RenderSize;
use vello_hybrid::{RenderTargetConfig, Renderer};
use wgpu::TextureFormat;
use winit::window::Window;

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

// This enum is only a few hundred bytes.
#[allow(clippy::large_enum_variant)]
enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended,
}

pub struct VelloHybridRenderer<'a> {
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
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {
    scene.set_paint(Paint::from(fill_color));
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
        self.scene = Scene::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn prepare_render_list(&mut self, render_list: RenderList, resource_manager: Arc<ResourceManager>, window: Rectangle) {
        
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            let scene = &mut self.scene;

            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, *rectangle, *fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    self.scene.set_stroke(vello_common::kurbo::Stroke::new(1.0));
                    self.scene.set_paint(Paint::from(*outline_color));
                    self.scene.stroke_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawImage(_rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(resource_identifier);

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
                RenderCommand::DrawText(text_render, rect, text_scroll, show_cursor) => {
                    let text_transform =
                        kurbo::Affine::default().with_translation(kurbo::Vec2::new(rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let mut skip_remaining_lines = false;
                    let mut skip_line = false;
                    for line in &text_render.lines {
                        if skip_remaining_lines {
                            break;
                        }
                        if skip_line {
                            skip_line = false;
                            continue;
                        }
                        for item in &line.items {
                            if let Some(first_glyph) = item.glyphs.first() {
                                // Cull the selections vertically that are outside the window
                                let gy = first_glyph.y + rect.y - scroll;
                                if gy < window.y {
                                    skip_line = true;
                                    break;
                                } else if gy > (window.y + window.height) {
                                    skip_remaining_lines = true;
                                    break;
                                }
                            }

                            for selection in &line.selections {
                                let selection_rect = Rectangle {
                                    x: selection.x + rect.x,
                                    y: -scroll + selection.y + rect.y,
                                    width: selection.width,
                                    height: selection.height,
                                };
                                vello_draw_rect(scene, selection_rect, Color::from_rgb8(0, 120, 215));
                            }
                        }
                    }
                    skip_remaining_lines = false;
                    skip_line = false;
                    for line in &text_render.lines {
                        if skip_remaining_lines {
                            break;
                        }
                        if skip_line {
                            skip_line = false;
                            continue;
                        }
                        for item in &line.items {
                            if let Some(first_glyph) = item.glyphs.first() {
                                // Cull the glyphs vertically that are outside the window
                                let gy = first_glyph.y + rect.y - scroll;
                                if gy < window.y {
                                    skip_line = true;
                                    break;
                                } else if gy > (window.y + window.height) {
                                    skip_remaining_lines = true;
                                    break;
                                }
                            }

                            scene.set_paint(Paint::from(text_render.override_brush.map(|b| b.color).unwrap_or_else(|| item.brush.color)));
                            scene.reset_transform();

                            let glyph_run_builder = scene
                                .glyph_run(&item.font)
                                .font_size(item.font_size)
                                .glyph_transform(text_transform);
                            glyph_run_builder.fill_glyphs(item.glyphs.iter().map(|glyph| Glyph {
                                id: glyph.id as u32,
                                x: glyph.x,
                                y: glyph.y,
                            }));
                        }
                    }
                    if *show_cursor {
                        if let Some(cursor) = &text_render.cursor {
                            let cursor_rect = Rectangle {
                                x: cursor.x + rect.x,
                                y: -scroll + cursor.y + rect.y,
                                width: cursor.width,
                                height: cursor.height,
                            };
                            vello_draw_rect(scene, cursor_rect, Color::from_rgb8(0, 0, 0));
                        }
                    }
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color) => {
                    draw_tiny_vg(scene, *rectangle, &resource_manager, resource_identifier.clone(), override_color);
                }
                RenderCommand::PushLayer(rect) => {
                    let clip_path = Some(peniko::kurbo::Rect::from_origin_size(peniko::kurbo::Point::new(rect.x as f64, rect.y as f64), peniko::kurbo::Size::new(rect.width as f64, rect.height as f64)).into_path(0.1));
                    scene.push_layer(clip_path.as_ref(), None, None, None);
                }
                RenderCommand::PopLayer => {
                    scene.pop_layer();
                }
                RenderCommand::FillBezPath(path, brush) => {
                    scene.set_paint(brush_to_paint(brush));
                    scene.fill_path(path);
                }
                _ => {}
            }
        });
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

        let mut encoder = device_handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Vello Render to Surface pass"),
        });
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render(
                &self.scene,
                &device_handle.device,
                &device_handle.queue,
                &mut encoder,
                &render_size,
                &texture_view,
            )
            .unwrap();

        device_handle.queue.submit([encoder.finish()]);

        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}

fn brush_to_paint(brush: &Brush) -> Paint {
    match brush {
        Brush::Color(color) => {
            Paint::from(*color)
        }
        Brush::Gradient(gradient) => {
            // Paint::Gradient does not exist yet, so we need to come back and fix this later.
            let color = gradient.stops.first().map(|c| c.color.to_alpha_color()).unwrap_or(Color::BLACK);
            Paint::from(color)
        }
    }
}