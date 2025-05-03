pub(crate) mod tinyvg;

use crate::renderer::renderer::{RenderList, Renderer, SortedCommands, TextScroll};
use crate::renderer::vello_cpu::tinyvg::draw_tiny_vg;
use crate::renderer::{Brush, RenderCommand};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::ResourceManager;
use cosmic_text::FontSystem;
use peniko::kurbo::{Affine, Rect};
use peniko::{kurbo, BlendMode, Color, Compose, Fill, Mix};
use softbuffer::Buffer;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use vello_common::glyph::Glyph;
use vello_common::kurbo::Stroke;
use vello_common::paint::PaintType;
use vello_cpu::{Pixmap, RenderContext};
use winit::window::Window;
use peniko::kurbo::Shape;

pub struct Surface {
    inner_surface: softbuffer::Surface<Arc<dyn Window>, Arc<dyn Window>>,
}

impl Surface {
    // Constructor for the SurfaceWrapper
    pub fn new(window: Arc<dyn Window>) -> Self {
        let context = softbuffer::Context::new(window.clone()).expect("Failed to create softbuffer context");
        Self {
            inner_surface: softbuffer::Surface::new(&context, window.clone()).expect("Failed to create surface"),
        }
    }
}

// Implement Deref to expose all methods from the inner Surface
impl Deref for Surface {
    type Target = softbuffer::Surface<Arc<dyn Window>, Arc<dyn Window>>;

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

pub(crate) struct VelloCpuRenderer {
    window: Arc<dyn Window>,
    render_context: RenderContext,
    pixmap: Pixmap,
    surface: Surface,
    clear_color: Color,
}

impl VelloCpuRenderer {
    pub fn new(window: Arc<dyn Window>) -> Self {
        let width = window.surface_size().width as u16;
        let height = window.surface_size().height as u16;

        let render_context = RenderContext::new(width, height);

        let pixmap = Pixmap::new(width, height);

        let mut surface = Surface::new(window.clone());
        surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");

        Self {
            window,
            render_context,
            pixmap,
            surface,
            clear_color: Color::WHITE,
        }
    }
}

impl Renderer for VelloCpuRenderer {
    fn surface_width(&self) -> f32 {
        self.window.surface_size().width as f32
    }

    fn surface_height(&self) -> f32 {
        self.window.surface_size().height as f32
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let width = width.max(1.0);
        let height = height.max(1.0);
        self.surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        self.pixmap = Pixmap::new(width as u16, height as u16);
        self.render_context = RenderContext::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn prepare_render_list(
        &mut self,
        render_list: RenderList,
        resource_manager: Arc<ResourceManager>,
        font_system: &mut FontSystem,
    ) {
        let paint = PaintType::Solid(self.clear_color);
        self.render_context.set_paint(paint);
        self.render_context.set_fill_rule(Fill::NonZero);
        self.render_context.set_transform(Affine::IDENTITY);
        self.render_context.fill_rect(&Rect::new(0.0, 0.0, self.pixmap.width as f64, self.pixmap.height as f64));

        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    self.render_context.set_paint(PaintType::Solid(*fill_color));
                    self.render_context.fill_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    self.render_context.set_stroke(Stroke::new(1.0));
                    self.render_context.set_paint(PaintType::Solid(*outline_color));
                    self.render_context.stroke_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(resource) = resource {
                        if let Resource::Image(resource) = resource.as_ref() {
                            let image = &resource.image;
                            for (x, y, pixel) in image.enumerate_pixels() {
                                let color = Color::from_rgba8(pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
                                self.render_context.set_paint(PaintType::Solid(color));
                                let pixel = Rect::new(
                                    rectangle.x as f64 + x as f64,
                                    rectangle.y as f64 + y as f64,
                                    rectangle.x as f64 + x as f64 + 1.0,
                                    rectangle.y as f64 + y as f64 + 1.0,
                                );
                                self.render_context.fill_rect(&pixel);
                            }
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
                            self.render_context.set_paint(PaintType::Solid(
                                buffer_glyphs.glyph_highlight_color
                            ));
                            self.render_context.set_transform(text_transform);
                            self.render_context.fill_rect(glyph_highlight);
                        }

                        if *show_cursor {
                            if let Some(cursor) = &buffer_line.cursor {
                                self.render_context
                                    .set_paint(PaintType::Solid(buffer_glyphs.cursor_color));
                                self.render_context.set_transform(text_transform);
                                self.render_context.fill_rect(cursor);
                            }
                        }

                        for glyph_run in &buffer_line.glyph_runs {
                            let font = font_system.get_font(glyph_run.font).unwrap().as_peniko();
                            let glyph_color = glyph_run.glyph_color;
                            let glyphs = glyph_run.glyphs.clone();
                            self.render_context.set_paint(PaintType::Solid(glyph_color));
                            self.render_context.reset_transform();
                            let glyph_run_builder = self
                                .render_context
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
                RenderCommand::PushLayer(rect) => {
                    let clip_path = Some(peniko::kurbo::Rect::from_origin_size(peniko::kurbo::Point::new(rect.x as f64, rect.y as f64), peniko::kurbo::Size::new(rect.width as f64, rect.height as f64)).into_path(0.1));
                    self.render_context.push_layer(clip_path.as_ref(), None, None, None);
                }
                RenderCommand::PopLayer => {
                    self.render_context.pop_layer();
                }
                RenderCommand::FillBezPath(path, brush) => {
                    self.render_context.set_paint(brush_to_paint(&brush));
                    self.render_context.fill_path(&path);
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color) => {
                    draw_tiny_vg(&mut self.render_context, *rectangle, &resource_manager, resource_identifier.clone(), override_color);
                }
                _ => {}
            }
        });
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
        self.render_context.render_to_pixmap(&mut self.pixmap);
        let buffer = self.copy_pixmap_to_softbuffer(self.pixmap.width as usize, self.pixmap.height as usize);
        buffer.present().expect("Failed to present buffer");
        self.render_context.reset();
    }
}

impl VelloCpuRenderer {
    fn copy_pixmap_to_softbuffer(&mut self, width: usize, height: usize) -> Buffer<Arc<dyn Window>, Arc<dyn Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();

        let pixmap = &self.pixmap.buf;

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

fn brush_to_paint(brush: &Brush) -> PaintType {
    match brush {
        Brush::Color(color) => {
            PaintType::Solid(*color)
        }
        Brush::Gradient(gradient) => {
            PaintType::Gradient(gradient.clone())
        }
    }
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}
