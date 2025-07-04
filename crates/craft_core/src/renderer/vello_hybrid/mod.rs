mod render_context;
mod tinyvg;

use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, RenderList, Renderer as CraftRenderer, SortedCommands, TextScroll};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use chrono::{DateTime, Utc};
use kurbo::{Affine, Stroke};
use peniko::kurbo::Shape;
use peniko::ImageQuality;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use vello_common::color::PremulRgba8;
use vello_common::glyph::Glyph;
use vello_common::paint::ImageId;
use vello_common::paint::{ImageSource, PaintType};
use vello_common::pixmap::Pixmap;
use vello_common::{kurbo, peniko};
use vello_hybrid::RenderSize;
use vello_hybrid::{RenderTargetConfig, Renderer};
use wgpu::TextureFormat;
use winit::window::Window;

use crate::renderer::vello_hybrid::render_context::RenderContext;
use crate::renderer::vello_hybrid::render_context::RenderSurface;
use crate::renderer::vello_hybrid::tinyvg::draw_tiny_vg;
use crate::renderer::Brush;
use crate::text::text_render_data::TextRenderLine;
use vello_hybrid::Scene;

pub struct ActiveRenderState {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'static>,
    window_width: f32,
    window_height: f32,
}

// This enum is only a few hundred bytes.
#[allow(clippy::large_enum_variant)]
enum RenderState {
    Active(ActiveRenderState),
    Suspended,
}

pub struct VelloHybridRenderer {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc.)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,

    images: HashMap<ResourceIdentifier, (ImageId, Option<DateTime<Utc>>)>,
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

impl VelloHybridRenderer {
    pub(crate) async fn new(window: Arc<Window>) -> VelloHybridRenderer {
        // Create a vello Surface
        let surface_size = window.inner_size();

        let width = surface_size.width.max(1);
        let height = surface_size.height.max(1);

        let mut vello_renderer = VelloHybridRenderer {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended,
            scene: Scene::new(width as u16, height as u16),
            surface_clear_color: Color::WHITE,
            images: HashMap::new(),
        };

        let surface = vello_renderer
            .context
            .create_surface(
                window.clone(),
                width,
                height,
                wgpu::PresentMode::AutoVsync,
                #[cfg(feature = "vello_hybrid_renderer_webgl")]
                TextureFormat::Rgba8Unorm,
                #[cfg(not(feature = "vello_hybrid_renderer_webgl"))]
                TextureFormat::Bgra8Unorm,
            )
            .await;

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState {
            surface,
            window_width: width as f32,
            window_height: height as f32,
        });

        vello_renderer
    }
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {
    scene.set_paint(PaintType::from(fill_color));
    scene.fill_rect(&rectangle.to_kurbo());
}

impl CraftRenderer for VelloHybridRenderer {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window_width,
            RenderState::Suspended => 0.0,
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window_height,
            RenderState::Suspended => 0.0,
        }
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };
        render_state.window_height = height;
        render_state.window_width = width;
        self.context.resize_surface(&mut render_state.surface, width as u32, height as u32);
        self.scene = Scene::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn prepare_render_list(
        &mut self,
        render_list: RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    ) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };

        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        vello_draw_rect(&mut self.scene, Rectangle::new(0.0, 0.0, width as f32, height as f32), Color::WHITE);

        let renderer = self.renderers[surface.dev_id]
            .as_mut()
            .unwrap();
        let device_handle = &self.context.devices[surface.dev_id];
        let mut encoder = device_handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Blit Textures onto a Texture Atlas Encoder"),
        });

        let mut images_were_uploaded = false;

        let mut seen_images: HashSet<ImageId> = HashSet::new();
        let mut expired_images: HashSet<ImageId> = HashSet::new();
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            let scene = &mut self.scene;

            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, *rectangle, *fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    self.scene.set_stroke(Stroke::new(1.0));
                    self.scene.set_paint(PaintType::from(*outline_color));
                    self.scene.stroke_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(resource_identifier);

                    if let Some(resource) = resource && let Resource::Image(resource) = resource.as_ref() {
                        let expiration_time = resource.common_data.expiration_time;

                        let image = &resource.image;

                        // There is an image, and it hasn't expired.
                        let image_id = if let Some(stored_image) = self.images.get(resource_identifier) && stored_image.1 == expiration_time {
                            stored_image.0
                        } else {
                            // There is an image, but it expired.
                            if let Some(stored_image) = self.images.get(resource_identifier) {
                                expired_images.insert(stored_image.0);
                            }

                            images_were_uploaded = true;

                            // NOTE: We may be able to avoid this if we implement the AtlasWriter trait.
                            let premul_data: Vec<PremulRgba8> = image
                                .to_vec()
                                .chunks_exact(4)
                                .map(|rgba| {
                                    let alpha = u16::from(rgba[3]);
                                    let premultiply = |component| (alpha * (u16::from(component)) / 255) as u8;
                                    PremulRgba8 {
                                        r: premultiply(rgba[0]),
                                        g: premultiply(rgba[1]),
                                        b: premultiply(rgba[2]),
                                        a: alpha as u8,
                                    }
                                })
                                .collect();
                            let pixmap = Pixmap::from_parts(
                                premul_data,
                                image.width() as u16,
                                image.height() as u16,
                            );

                            let image_id = renderer.upload_image(&device_handle.device, &device_handle.queue, &mut encoder, &pixmap);
                            self.images.insert(resource_identifier.clone(), (image_id, expiration_time));

                            image_id
                        };
                        seen_images.insert(image_id);

                        let mut transform = Affine::IDENTITY;
                        transform =
                            transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            rectangle.width as f64 / image.width() as f64,
                            rectangle.height as f64 / image.height() as f64,
                        );
                        scene.set_transform(transform);

                        scene.set_paint(PaintType::Image(vello_common::paint::Image {
                            source: ImageSource::OpaqueId(image_id),
                            x_extend: peniko::Extend::default(),
                            y_extend: peniko::Extend::default(),
                            quality: ImageQuality::High,
                        }));

                        scene.fill_rect(&kurbo::Rect::new(
                            0.0,
                            0.0,
                            image.width() as f64,
                            image.height() as f64,
                        ));
                        scene.reset_transform();
                    }
                }
                RenderCommand::DrawText(text_render, rect, text_scroll, show_cursor) => {
                    let text_transform =
                        Affine::default().with_translation(kurbo::Vec2::new(rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let cull_and_process = |process_line: &mut dyn FnMut(&TextRenderLine)| {
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
                            }

                            process_line(line);
                        }
                    };


                    cull_and_process(&mut |line: &TextRenderLine| {
                        for (background, color) in &line.backgrounds {
                            let background_rect = Rectangle {
                                x: background.x + rect.x,
                                y: -scroll + background.y + rect.y,
                                width: background.width,
                                height: background.height,
                            };
                            vello_draw_rect(scene, background_rect, *color);
                        }
                        
                        for (selection, selection_color) in &line.selections {
                            let selection_rect = Rectangle {
                                x: selection.x + rect.x,
                                y: -scroll + selection.y + rect.y,
                                width: selection.width,
                                height: selection.height,
                            };
                            vello_draw_rect(scene, selection_rect, *selection_color);
                        }
                    });

                    cull_and_process(&mut |line: &TextRenderLine| {
                        for item in &line.items {
                            if let Some(underline) = &item.underline {
                                scene.set_transform(text_transform);
                                scene.set_stroke(Stroke::new(underline.width.into()));
                                scene.set_paint(PaintType::from(underline.brush.color));
                                scene.stroke_path(&underline.line.to_path(0.1));
                            }

                            scene.set_paint(PaintType::from(
                                text_render.override_brush.map(|b| b.color).unwrap_or_else(|| item.brush.color),
                            ));
                            scene.reset_transform();

                            let glyph_run_builder =
                                scene.glyph_run(&item.font).font_size(item.font_size).glyph_transform(text_transform);
                            glyph_run_builder.fill_glyphs(item.glyphs.iter().map(|glyph| Glyph {
                                id: glyph.id as u32,
                                x: glyph.x,
                                y: glyph.y,
                            }));
                        }
                    });

                    if *show_cursor {
                        if let Some((cursor, cursor_color)) = &text_render.cursor {
                            let cursor_rect = Rectangle {
                                x: cursor.x + rect.x,
                                y: -scroll + cursor.y + rect.y,
                                width: cursor.width,
                                height: cursor.height,
                            };
                            vello_draw_rect(scene, cursor_rect, *cursor_color);
                        }
                    }
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color) => {
                    draw_tiny_vg(scene, *rectangle, &resource_manager, resource_identifier.clone(), override_color);
                }
                RenderCommand::PushLayer(rect) => {
                    let clip_path = Some(
                        kurbo::Rect::from_origin_size(
                            kurbo::Point::new(rect.x as f64, rect.y as f64),
                            kurbo::Size::new(rect.width as f64, rect.height as f64),
                        )
                        .into_path(0.1),
                    );
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

        let mut to_remove: Vec<ResourceIdentifier> = Vec::new();

        for expired_image_id in &expired_images {
            // Note: Expired images will have an entry in the images hashmap, but with a new ImageId.
            // Meaning, that we need to delete the detached/abandoned expired image id here.
            renderer.destroy_image(&device_handle.device, &device_handle.queue, &mut encoder, *expired_image_id);
            images_were_uploaded = true;
        }

        for (key, (image_id, _)) in &self.images {
            let seen = seen_images.contains(image_id);
            let expired = expired_images.contains(image_id);

            // Delete the culled image, only if it hasn't already been deleted when we looped over the expired images.
            if !seen && !expired {
                renderer.destroy_image(&device_handle.device, &device_handle.queue, &mut encoder, *image_id);
                images_were_uploaded = true;
            }

            if !seen {
                to_remove.push(key.clone());
            }
        }
        for key in &to_remove {
            self.images.remove(key);
        }

        // Submit the texture write commands.
        if images_were_uploaded {
            device_handle.queue.submit([encoder.finish()]);
        }
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
            .render(&self.scene, &device_handle.device, &device_handle.queue, &mut encoder, &render_size, &texture_view)
            .unwrap();

        device_handle.queue.submit([encoder.finish()]);

        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}

fn brush_to_paint(brush: &Brush) -> PaintType {
    match brush {
        Brush::Color(color) => PaintType::from(*color),
        Brush::Gradient(gradient) => {
            // Paint::Gradient does not exist yet, so we need to come back and fix this later.
            let color = gradient.stops.first().map(|c| c.color.to_alpha_color()).unwrap_or(Color::BLACK);
            PaintType::from(color)
        }
    }
}
