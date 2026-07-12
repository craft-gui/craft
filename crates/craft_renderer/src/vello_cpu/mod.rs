use std::any::Any;
use std::num::{NonZero, NonZeroU32};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use craft_primitives::geometry::{Rectangle, TOLERANCE};
use craft_resource_manager::ResourceManager;

use peniko::kurbo::{Affine, Shape};
use peniko::{BlendMode, Blob, Color, Compose, Fill, ImageAlphaType, Mix, kurbo};

use softbuffer::Buffer;

use glifo::Glyph;
use vello_common::filter_effects::{Filter, FilterFunction};
use vello_common::kurbo::Stroke;
use vello_common::paint::PaintType;
use vello_cpu::{Pixmap, RenderContext, Resources};

use winit::window::Window;
use craft_resource_manager::image::ImageResource;
use crate::RenderCommand;
use crate::helpers::{brush_to_paint, rgba_to_encoded_u32};
use crate::image_adapter::ImageAdapter;
use crate::render_command::{BoxShadowCmd, DrawRectCmd, PushLayerCmd};
use crate::render_list::RenderList;
use crate::renderer::Renderer;
use crate::screenshot::Screenshot;
use crate::sort_commands::SortedCommands;
use crate::text_renderer_data::{TextRenderLine, TextScroll};

pub(crate) struct VelloCpuRenderer {
    render_context: RenderContext,
    pixmap: Pixmap,
    surface: Surface,
    clear_color: Color,
    window_width: u16,
    window_height: u16,
    resources: Resources,
    render_list: RenderList,
}

pub struct Surface {
    inner_surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
}

impl Surface {
    // Constructor for the SurfaceWrapper
    pub fn new(window: Arc<Window>) -> Self {
        let context = softbuffer::Context::new(window.clone()).expect("Failed to create softbuffer context");
        Self {
            inner_surface: softbuffer::Surface::new(&context, window.clone()).expect("Failed to create surface"),
        }
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for Surface {}

// Implement Deref to expose all methods from the inner Surface
impl Deref for Surface {
    type Target = softbuffer::Surface<Arc<Window>, Arc<Window>>;

    fn deref(&self) -> &Self::Target {
        &self.inner_surface
    }
}

// Implement DerefMut to expose mutable methods from the inner Surface
impl DerefMut for Surface {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_surface
    }
}

fn vello_draw_rect(scene: &mut RenderContext, cmd: DrawRectCmd) {
    scene.set_transform(cmd.transform);
    scene.set_paint(PaintType::from(cmd.color));
    scene.fill_rect(&cmd.rect.to_kurbo());
}

pub fn draw_box_shadow(scene: &mut RenderContext, cmd: &BoxShadowCmd) {
    let radius = cmd.box_shadow.blur_radius / 2.0;
    let filter = Some(Filter::from_function(FilterFunction::Blur {
        radius: cmd.box_shadow.blur_radius as f32,
    }));

    if cmd.box_shadow.inset {
        scene.set_transform(cmd.transform);
        let mut clip_path = kurbo::BezPath::new();
        let outline_rect = cmd.box_shadow.border_box.expand((radius * 3.0) as f32).to_kurbo();
        clip_path.extend(&outline_rect.to_path(0.1));
        clip_path.extend(&cmd.box_shadow.path);
        scene.push_layer(Some(&cmd.box_shadow.outline), None, None, None, filter);
        scene.set_fill_rule(Fill::EvenOdd);
        scene.set_paint(cmd.box_shadow.color);
        scene.fill_path(&clip_path);
        scene.pop_layer();
        scene.set_fill_rule(Fill::NonZero);
    } else {
        scene.push_layer(
            None,
            Some(BlendMode::new(Mix::Normal, Compose::SrcOver)),
            None,
            None,
            filter,
        );

        scene.set_transform(cmd.transform * Affine::translate(cmd.box_shadow.offset));
        scene.set_paint(cmd.box_shadow.color);
        scene.fill_path(&cmd.box_shadow.path);
        scene.set_transform(cmd.transform);

        scene.set_blend_mode(BlendMode::new(Mix::Normal, Compose::DestOut));
        scene.set_paint(Color::WHITE);
        scene.fill_path(&cmd.box_shadow.outline);
        scene.set_blend_mode(BlendMode::new(Mix::Normal, Compose::SrcOver));

        scene.pop_layer();
    }
}

impl VelloCpuRenderer {
    pub fn new(window: Arc<Window>) -> Self {
        let width = window.inner_size().width as u16;
        let height = window.inner_size().height as u16;

        let render_context = RenderContext::new(width, height);

        let pixmap = Pixmap::new(width, height);

        let mut surface = Surface::new(window.clone());
        surface
            .resize(
                NonZeroU32::new(width as u32).unwrap_or(NonZero::new(1).unwrap()),
                NonZeroU32::new(height as u32).unwrap_or(NonZero::new(1).unwrap()),
            )
            .expect("TODO: panic message");

        Self {
            render_context,
            pixmap,
            surface,
            clear_color: Color::WHITE,
            window_width: width,
            window_height: height,
            resources: Resources::new(),
            render_list: Default::default(),
        }
    }
}

impl Renderer for VelloCpuRenderer {
    fn surface_width(&self) -> f32 {
        self.window_width as f32
    }

    fn surface_height(&self) -> f32 {
        self.window_height as f32
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let width = width.max(1.0);
        let height = height.max(1.0);
        self.window_width = width as u16;
        self.window_height = height as u16;
        self.surface
            .resize(
                NonZeroU32::new(width as u32).unwrap(),
                NonZeroU32::new(height as u32).unwrap(),
            )
            .expect("TODO: panic message");
        self.pixmap = Pixmap::new(width as u16, height as u16);
        self.render_context = RenderContext::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn render_list(&self) -> &RenderList {
        &self.render_list
    }

    fn render_list_mut(&mut self) -> &mut RenderList {
        &mut self.render_list
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(
        &mut self,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    ) {
        vello_draw_rect(
            &mut self.render_context,
            DrawRectCmd {
                rect: Rectangle::new(0.0, 0.0, self.window_width as f32, self.window_height as f32),
                color: self.clear_color,
                transform: Affine::IDENTITY
            },
        );

        let paint = PaintType::Solid(Color::WHITE);
        self.render_context.set_paint(paint);
        self.render_context.set_fill_rule(Fill::NonZero);
        self.render_context.set_transform(Affine::IDENTITY);

        let render_list = &self.render_list;
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            match command {
                RenderCommand::DrawRect(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context.set_paint(PaintType::Solid(cmd.color));
                    self.render_context.fill_rect(&cmd.rect.to_kurbo());
                }
                RenderCommand::DrawRectOutline(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context.set_stroke(Stroke::new(cmd.thickness));
                    self.render_context.set_paint(PaintType::Solid(cmd.outline_color));
                    self.render_context.stroke_rect(&cmd.rect.to_kurbo());
                }
                RenderCommand::DrawImage(cmd) => {
                    let resource = resource_manager.get(&cmd.resource_id);

                    if let Some(resource) = resource
                        && resource.resource_type == "image" && let Some(image) = resource.data.downcast_ref::<ImageResource>()
                    {
                        let arc = Arc::new(image.clone());
                        let data = Arc::new(ImageAdapter::new(arc));
                        let blob = Blob::new(data);
                        let id = peniko::ImageData {
                            data: blob,
                            format: peniko::ImageFormat::Rgba8,
                            alpha_type: ImageAlphaType::Alpha,
                            width: image.image.width(),
                            height: image.image.height(),
                        };

                        let mut transform = Affine::IDENTITY;
                        transform = transform.with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            cmd.rect.width as f64 / image.image.width() as f64,
                            cmd.rect.height as f64 / image.image.height() as f64,
                        );
                        self.render_context.set_transform(cmd.transform * transform);

                        let is = vello_common::paint::ImageSource::from_peniko_image_data(&id);
                        let id = vello_common::paint::Image {
                            image: is,
                            sampler: Default::default(),
                        };

                        self.render_context.set_paint(PaintType::Image(id));
                        self.render_context.fill_rect(&kurbo::Rect::new(
                            0.0,
                            0.0,
                            image.image.width() as f64,
                            image.image.height() as f64,
                        ));
                    }
                }
                RenderCommand::DrawText(cmd) => {
                    let text_transform = Affine::default()
                        .with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
                    let scroll = cmd.text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));
                    let transformed_container = Rectangle::from_kurbo(cmd.transform.transform_rect_bbox(cmd.rect.to_kurbo()));

                    let c = cmd.data.upgrade();
                    if c.is_none() {
                        return;
                    }
                    let c = c.unwrap();
                    let c = c.borrow();
                    let text_render = c.get_text_renderer().expect("Text render not found");

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
                                    let gy = first_glyph.y + transformed_container.y - scroll;
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
                                x: background.x + cmd.rect.x,
                                y: -scroll + background.y + cmd.rect.y,
                                width: background.width,
                                height: background.height,
                            };
                            vello_draw_rect(&mut self.render_context, DrawRectCmd {
                                rect: background_rect,
                                color: *color,
                                transform: cmd.transform
                            });
                        }

                        for (selection, selection_color) in &line.selections {
                            let selection_rect = Rectangle {
                                x: selection.x + cmd.rect.x,
                                y: -scroll + selection.y + cmd.rect.y,
                                width: selection.width,
                                height: selection.height,
                            };
                            vello_draw_rect(&mut self.render_context, DrawRectCmd {
                                rect: selection_rect,
                                color: *selection_color,
                                transform: cmd.transform
                            });
                        }
                    });

                    self.render_context.set_transform(cmd.transform * text_transform);

                    cull_and_process(&mut |line: &TextRenderLine| {
                        for item in &line.items {
                            if let Some(underline) = &item.underline {
                                self.render_context.set_stroke(Stroke::new(underline.width.into()));
                                self.render_context.set_paint(PaintType::from(underline.brush.color));
                                self.render_context.stroke_path(&underline.line.to_path(0.1));
                            }

                            self.render_context.set_paint(PaintType::from(
                                text_render
                                    .override_brush
                                    .map(|b| b.color)
                                    .unwrap_or_else(|| item.brush.color),
                            ));

                            let glyph_run_builder = self
                                .render_context
                                .glyph_run(&mut self.resources, &item.font)
                                //.atlas_cache(true)
                                .font_size(item.font_size);
                            glyph_run_builder.fill_glyphs(item.glyphs.iter().map(|glyph| Glyph {
                                id: glyph.id,
                                x: glyph.x,
                                y: glyph.y,
                            }));
                        }
                    });

                    if cmd.show_cursor
                        && let Some((cursor, cursor_color)) = &text_render.cursor
                    {
                        let cursor_rect = Rectangle {
                            x: cursor.x + cmd.rect.x,
                            y: -scroll + cursor.y + cmd.rect.y,
                            width: cursor.width,
                            height: cursor.height,
                        };
                        vello_draw_rect(&mut self.render_context, DrawRectCmd {
                            rect: cursor_rect,
                            color: *cursor_color,
                            transform: cmd.transform
                        });
                    }
                }
                RenderCommand::PushLayer(cmd) => {
                    match cmd {
                        PushLayerCmd::BezPath(path, transform) => {
                            self.render_context.set_transform(*transform);
                            self.render_context.push_layer(Some(path), None, None, None, None);
                        },
                        PushLayerCmd::Rect(rect, transform) => {
                            self.render_context.set_transform(*transform);
                            let clip_path = &rect.to_kurbo().into_path(0.1);
                            self.render_context.push_layer(Some(clip_path), None, None, None, None);
                        },
                    };
                }
                RenderCommand::PopLayer => {
                    self.render_context.pop_layer();
                }
                RenderCommand::FillBezPath(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context.set_paint(brush_to_paint(&cmd.brush));
                    self.render_context.fill_path(&cmd.path);
                }
                RenderCommand::StartOverlay => {}
                RenderCommand::EndOverlay => {}
                RenderCommand::BoxShadowCmd(cmd) => draw_box_shadow(&mut self.render_context, cmd),
                RenderCommand::DrawCircleOutline(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context.set_stroke(Stroke::new(cmd.thickness as f64));
                    self.render_context.set_paint(PaintType::Solid(cmd.outline_color));
                    self.render_context
                        .stroke_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
                }
                RenderCommand::DrawCircle(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context.set_paint(PaintType::from(cmd.color));
                    self.render_context.fill_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
                }
                RenderCommand::StrokeBezPath(cmd) => {
                    self.render_context.set_transform(cmd.transform);
                    self.render_context
                        .set_paint(PaintType::from(brush_to_paint(&cmd.brush)));
                    self.render_context.stroke_path(&cmd.path);
                }
            }
        });
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
        self.render_context.flush();
        self.render_context.render(&mut self.pixmap, &mut self.resources);
        let buffer = self.copy_pixmap_to_softbuffer(self.pixmap.width() as usize, self.pixmap.height() as usize);
        buffer.present().expect("Failed to present buffer");
        self.render_context.reset();
    }

    fn screenshot(&self) -> Screenshot {
        Screenshot {
            width: self.pixmap.width(),
            height: self.pixmap.height(),
            pixels: self.pixmap.data_as_u8_slice().to_vec(),
        }
    }
}

impl VelloCpuRenderer {
    fn copy_pixmap_to_softbuffer(&mut self, width: usize, height: usize) -> Buffer<'_, Arc<Window>, Arc<Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();

        let pixmap = &self.pixmap.data_as_u8_slice();

        for offset in 0..(width * height) {
            let red = pixmap[4 * offset];
            let green = pixmap[4 * offset + 1];
            let blue = pixmap[4 * offset + 2];
            let alpha = pixmap[4 * offset + 3];

            buffer[offset] = rgba_to_encoded_u32(red as u32, green as u32, blue as u32, alpha as u32);
        }

        buffer
    }
}
