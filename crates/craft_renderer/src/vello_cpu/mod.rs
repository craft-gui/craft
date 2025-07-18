pub(crate) mod tinyvg;

use std::any::Any;
use craft_primitives::geometry::Rectangle;
use crate::image_adapter::ImageAdapter;
use crate::renderer::{RenderList, Renderer, SortedCommands, TextScroll};
use crate::vello_cpu::tinyvg::draw_tiny_vg;
use crate::{Brush, RenderCommand};
use craft_resource_manager::resource::Resource;
use craft_resource_manager::ResourceManager;
use peniko::kurbo::Affine;
use peniko::kurbo::Shape;
use peniko::{kurbo, Blob, Color, Fill};
use softbuffer::Buffer;
use std::num::NonZero;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use vello_common::glyph::Glyph;
use vello_common::kurbo::Stroke;
use vello_common::paint::PaintType;
use vello_cpu::{Pixmap, RenderContext, RenderMode};
use winit::window::Window;
use crate::text_renderer_data::{TextRender, TextRenderLine};

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

fn vello_draw_rect(scene: &mut RenderContext, rectangle: Rectangle, fill_color: Color) {
    scene.set_paint(PaintType::from(fill_color));
    scene.fill_rect(&rectangle.to_kurbo());
}

pub(crate) struct VelloCpuRenderer {
    render_context: RenderContext,
    pixmap: Pixmap,
    surface: Surface,
    clear_color: Color,
    window_width: u16,
    window_height: u16,
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
        }
    }
}

impl Renderer for VelloCpuRenderer {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
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
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        self.pixmap = Pixmap::new(width as u16, height as u16);
        self.render_context = RenderContext::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn prepare_render_list<'a>(
        &'a mut self,
        render_list: &'a mut RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
        get_text_renderer: Box<dyn Fn(u64) -> Option<&'a TextRender> + 'a>,
    ) {
        vello_draw_rect(&mut self.render_context, Rectangle::new(0.0, 0.0, self.window_width as f32, self.window_height as f32), Color::WHITE);
        
        let paint = PaintType::Solid(self.clear_color);
        self.render_context.set_paint(paint);
        self.render_context.set_fill_rule(Fill::NonZero);
        self.render_context.set_transform(Affine::IDENTITY);

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
                    let resource = resource_manager.resources.get(resource_identifier);

                    if let Some(resource) = resource && let Resource::Image(resource) = resource.as_ref() {
                        let image = &resource.image;
                        let data = Arc::new(ImageAdapter::new(resource.clone()));
                        let blob = Blob::new(data);
                        let vello_image =
                            peniko::Image::new(blob, peniko::ImageFormat::Rgba8, image.width(), image.height());

                        let mut transform = Affine::IDENTITY;
                        transform =
                            transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            rectangle.width as f64 / image.width() as f64,
                            rectangle.height as f64 / image.height() as f64,
                        );
                        self.render_context.set_transform(transform);
                        self.render_context.set_paint(PaintType::Image(
                            vello_common::paint::Image::from_peniko_image(&vello_image),
                        ));
                        self.render_context.fill_rect(&kurbo::Rect::new(
                            0.0,
                            0.0,
                            image.width() as f64,
                            image.height() as f64,
                        ));
                        self.render_context.reset_transform(); 
                    }
                }
                RenderCommand::DrawText(text_render, rect, text_scroll, show_cursor) => {
                    let text_transform =
                        kurbo::Affine::default().with_translation(kurbo::Vec2::new(rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let text_render = get_text_renderer(*text_render).expect("Text render not found");

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
                            vello_draw_rect(
                                &mut self.render_context,
                                background_rect,
                                *color,
                            );
                        }
                        
                        for (selection, selection_color) in &line.selections {
                            let selection_rect = Rectangle {
                                x: selection.x + rect.x,
                                y: -scroll + selection.y + rect.y,
                                width: selection.width,
                                height: selection.height,
                            };
                            vello_draw_rect(
                                &mut self.render_context,
                                selection_rect,
                                *selection_color,
                            );
                        }
                    });

                    cull_and_process(&mut |line: &TextRenderLine| {
                        for item in &line.items {
                            if let Some(underline) = &item.underline {
                                self.render_context.set_transform(text_transform);
                                self.render_context.set_stroke(Stroke::new(underline.width.into()));
                                self.render_context.set_paint(PaintType::from(underline.brush.color));
                                self.render_context.stroke_path(&underline.line.to_path(0.1));
                            }
                            
                            self.render_context.set_paint(PaintType::from(
                                text_render.override_brush.map(|b| b.color).unwrap_or_else(|| item.brush.color),
                            ));
                            self.render_context.reset_transform();

                            let glyph_run_builder = self
                                .render_context
                                .glyph_run(&item.font)
                                .font_size(item.font_size)
                                .glyph_transform(text_transform);
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
                            vello_draw_rect(&mut self.render_context, cursor_rect, *cursor_color);
                        }
                    }
                }
                RenderCommand::PushLayer(rect) => {
                    let clip_path = Some(
                        peniko::kurbo::Rect::from_origin_size(
                            peniko::kurbo::Point::new(rect.x as f64, rect.y as f64),
                            peniko::kurbo::Size::new(rect.width as f64, rect.height as f64),
                        )
                        .into_path(0.1),
                    );
                    self.render_context.push_layer(clip_path.as_ref(), None, None, None);
                }
                RenderCommand::PopLayer => {
                    self.render_context.pop_layer();
                }
                RenderCommand::FillBezPath(path, brush) => {
                    self.render_context.set_paint(brush_to_paint(brush));
                    self.render_context.fill_path(path);
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color) => {
                    draw_tiny_vg(
                        &mut self.render_context,
                        *rectangle,
                        &resource_manager,
                        resource_identifier.clone(),
                        override_color,
                    );
                }
                _ => {}
            }
        });
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
        self.render_context.flush();
        self.render_context.render_to_pixmap(&mut self.pixmap, RenderMode::OptimizeQuality);
        let buffer = self.copy_pixmap_to_softbuffer(self.pixmap.width() as usize, self.pixmap.height() as usize);
        buffer.present().expect("Failed to present buffer");
        self.render_context.reset();
    }
}

impl VelloCpuRenderer {
    fn copy_pixmap_to_softbuffer(&mut self, width: usize, height: usize) -> Buffer<Arc<Window>, Arc<Window>> {
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

fn brush_to_paint(brush: &Brush) -> PaintType {
    match brush {
        Brush::Color(color) => PaintType::Solid(*color),
        Brush::Gradient(gradient) => PaintType::Gradient(gradient.clone()),
    }
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}
