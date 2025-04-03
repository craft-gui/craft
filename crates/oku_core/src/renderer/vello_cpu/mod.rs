use crate::components::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::cosmic_adapter::CosmicFontBlobAdapter;
use crate::renderer::renderer::{Renderer, TextScroll};
use crate::renderer::{text, RenderCommand};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::color::PremulRgba8;
use peniko::kurbo::{Affine, BezPath, Rect};
use peniko::{kurbo, BlendMode, Blob, Color, Compose, Fill, Font, Mix};
use softbuffer::Buffer;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use vello_common::glyph::Glyph;
use vello_common::kurbo::Stroke;
use vello_common::paint::Paint;
use vello_cpu::{Pixmap, RenderContext};
use winit::window::Window;

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
    render_commands: Vec<RenderCommand>,
    window: Arc<dyn Window>,
    render_context: RenderContext,
    pixmap: vello_cpu::Pixmap,
    surface: Surface,
    clear_color: Color,
    vello_fonts: HashMap<cosmic_text::fontdb::ID, Font>,
}

impl VelloCpuRenderer {
    pub fn new(window: Arc<dyn Window>) -> Self {
        let width = window.surface_size().width as u16;
        let height = window.surface_size().height as u16;

        let render_context = vello_cpu::RenderContext::new(width, height);

        let pixmap = vello_cpu::Pixmap::new(width, height);

        let mut surface = Surface::new(window.clone());
        surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");

        Self {
            render_commands: Vec::new(),
            window,
            render_context,
            pixmap,
            surface,
            clear_color: Color::WHITE,
            vello_fonts: HashMap::new(),
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

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }
    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        self.render_commands.push(RenderCommand::DrawRectOutline(rectangle, outline_color));
    }

    fn fill_bez_path(&mut self, path: BezPath, color: Color) {
        self.render_commands.push(RenderCommand::FillBezPath(path, color));
    }

    fn draw_text(
        &mut self,
        element_id: ComponentId,
        rectangle: Rectangle,
        fill_color: Color,
        text_scroll: Option<TextScroll>,
    ) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color, text_scroll));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands
            .push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, _rect: Rectangle) {}

    fn pop_layer(&mut self) {}

    fn load_font(&mut self, font_system: &mut FontSystem) {
        let font_faces: Vec<(cosmic_text::fontdb::ID, u32)> =
            font_system.db().faces().map(|face| (face.id, face.index)).collect();
        for (font_id, index) in font_faces {
            if let Some(font) = font_system.get_font(font_id) {
                let font_blob = Blob::new(Arc::new(CosmicFontBlobAdapter::new(font)));
                let vello_font = Font::new(font_blob, index);
                self.vello_fonts.insert(font_id, vello_font);
            }
        }
    }

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        _font_system: &mut FontSystem,
        element_state: &ElementStateStore,
    ) {
        let paint = Paint::Solid(self.clear_color.premultiply().to_rgba8());
        self.render_context.set_paint(paint);
        self.render_context.set_blend_mode(BlendMode::new(Mix::Clip, Compose::SrcOver));
        self.render_context.set_fill_rule(Fill::NonZero);
        self.render_context.set_transform(Affine::IDENTITY);
        self.render_context.fill_rect(&Rect::new(0.0, 0.0, self.pixmap.width as f64, self.pixmap.height as f64));

        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    self.render_context.set_paint(Paint::Solid(fill_color.premultiply().to_rgba8()));
                    self.render_context.fill_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    self.render_context.set_stroke(Stroke::new(1.0));
                    self.render_context.set_paint(Paint::Solid(outline_color.premultiply().to_rgba8()));
                    self.render_context.stroke_rect(&rectangle.to_kurbo());
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        for (x, y, pixel) in image.enumerate_pixels() {
                            let premultiplied_color = PremulRgba8::from_u8_array(pixel.0);
                            self.render_context.set_paint(Paint::Solid(premultiplied_color));
                            let pixel = kurbo::Rect::new(
                                rectangle.x as f64 + x as f64,
                                rectangle.y as f64 + y as f64,
                                rectangle.x as f64 + x as f64 + 1.0,
                                rectangle.y as f64 + y as f64 + 1.0,
                            );
                            self.render_context.fill_rect(&pixel);
                        }
                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color, text_scroll) => {
                    let text_transform = Affine::translate((rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let mut draw_cursor = false;
                    let cached_editor = if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextState>()
                    {
                        Some(&text_context.cached_editor)
                    } else {
                        draw_cursor = true;
                        element_state
                            .storage
                            .get(&component_id)
                            .unwrap()
                            .data
                            .downcast_ref::<TextInputState>()
                            .map(|text_context| &text_context.cached_editor)
                    };

                    if let Some(cached_editor) = cached_editor {
                        let editor = &cached_editor.editor;
                        let buffer = &cached_editor.get_last_cache_entry().buffer;

                        let buffer_glyphs = text::create_glyphs_for_editor(
                            buffer,
                            editor,
                            fill_color,
                            Color::from_rgb8(0, 0, 0),
                            Color::from_rgb8(0, 120, 215),
                            Color::from_rgb8(255, 255, 255),
                            text_scroll,
                        );

                        // Draw the Glyphs
                        for buffer_line in &buffer_glyphs.buffer_lines {
                            for glyph_highlight in &buffer_line.glyph_highlights {
                                self.render_context.set_paint(Paint::Solid(buffer_glyphs.glyph_highlight_color.premultiply().to_rgba8()));
                                self.render_context.set_transform(text_transform);
                                self.render_context.fill_rect(glyph_highlight);
                            }

                            if draw_cursor {
                                if let Some(cursor) = &buffer_line.cursor {
                                    self.render_context.set_paint(Paint::Solid(buffer_glyphs.cursor_color.premultiply().to_rgba8()));
                                    self.render_context.set_transform(text_transform);
                                    self.render_context.fill_rect(cursor);
                                }
                            }

                            for glyph_run in &buffer_line.glyph_runs {
                                let font = self.vello_fonts.get(&glyph_run.font).unwrap();
                                let glyph_color = glyph_run.glyph_color;
                                let glyphs = glyph_run.glyphs.clone();
                                self.render_context.set_paint(Paint::Solid(glyph_color.premultiply().to_rgba8()));
                                self.render_context.reset_transform();
                                let glyph_run_builder = self
                                    .render_context
                                    .glyph_run(font)
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
                }
                RenderCommand::PushLayer(_rect) => {}
                RenderCommand::PopLayer => {}
                RenderCommand::FillBezPath(path, color) => {
                    self.render_context.set_paint(Paint::Solid(color.premultiply().to_rgba8()));
                    self.render_context.fill_path(&path);
                }
                #[cfg(feature = "wgpu_renderer")]
                RenderCommand::FillLyonPath(_, _) => {}
            }
        }
    }

    fn submit(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>) {
        self.render_context.render_to_pixmap(&mut self.pixmap);
        let buffer = self.copy_pixmap_to_softbuffer(self.pixmap.width as usize, self.pixmap.height as usize);
        buffer.present().expect("Failed to present buffer");
    }
}

impl VelloCpuRenderer {
    fn copy_pixmap_to_softbuffer(&mut self, width: usize, height: usize) -> Buffer<Arc<dyn Window>, Arc<dyn Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();

        let pixmap = &self.pixmap.buf;

        for offset in 0..(width * height) {
            let red = pixmap[4 * offset + 0];
            let green = pixmap[4 * offset + 1];
            let blue = pixmap[4 * offset + 2];
            let alpha = pixmap[4 * offset + 3];

            buffer[offset] = rgba_to_encoded_u32(red as u32, green as u32, blue as u32, alpha as u32);
        }

        buffer
    }
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}
