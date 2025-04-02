use cosmic_text::SwashCache;
use std::num::NonZeroU32;
use std::ops::DerefMut;
use std::ops::Deref;
use softbuffer::Buffer;
use crate::components::ComponentId;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::renderer::{Renderer, TextScroll};
use crate::renderer::{text, RenderCommand};
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo::{Affine, BezPath, Rect};
use peniko::{kurbo, BlendMode, Blob, Color, Compose, Fill, Mix};
use std::sync::Arc;
use peniko::color::PremulRgba8;
use tokio::sync::RwLockReadGuard;
use vello_common::paint::Paint;
use vello_cpu::{Pixmap, RenderContext};
use winit::window::Window;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::resource_manager::resource::Resource;

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
    cache: SwashCache,
}

impl VelloCpuRenderer {
    pub fn new(window: Arc<dyn Window>) -> Self {
        let width = window.surface_size().width as u16;
        let height = window.surface_size().height as u16;

        let render_context = vello_cpu::RenderContext::new(
            width,
            height,
        );

        let pixmap = vello_cpu::Pixmap::new(
            width,
            height,
        );

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
            cache: SwashCache::new(),
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
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
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
        self.render_commands.push(RenderCommand::DrawText(
            rectangle,
            element_id,
            fill_color,
            text_scroll,
        ));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
    }

    fn push_layer(&mut self, rect: Rectangle) {
    }

    fn pop_layer(&mut self) {
    }

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
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
                }
                RenderCommand::DrawRectOutline(_rectangle, _outline_color) => {
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                }
                RenderCommand::DrawText(rect, component_id, fill_color, text_scroll) => {
                    let fc = {
                        let [r, g, b, a] = fill_color.to_rgba8().to_u8_array();
                        cosmic_text::Color::rgba(r, g, b, a)
                    };
                    if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextInputState>()
                    {
                        let editor = &text_context.cached_editor.editor;
                        editor.draw(
                            font_system,
                            &mut self.cache,
                            cosmic_text::Color::rgba(0, 0, 0, 255),
                            cosmic_text::Color::rgba(0, 0, 0, 255),
                            cosmic_text::Color::rgba(0, 120, 215, 255),
                            cosmic_text::Color::rgba(255, 255, 255, 255),
                            |x, y, w, h, color: cosmic_text::Color| {
                                self.render_context.set_paint(Paint::Solid(PremulRgba8::from_u8_array(color.as_rgba())));
                                self.render_context.fill_rect(
                                    &Rect::new((rect.x + x as f32) as f64, (rect.y + y as f32) as f64, (rect.x + w as f32) as f64, (rect.y + h as f32) as f64)
                                )
                            },
                        );
                    } else if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextState>()
                    {
                        let buffer = &text_context.cached_editor.editor;
                        self.render_context.set_blend_mode(BlendMode::new(Mix::Clip, Compose::SrcOver));

                        buffer.draw(
                            font_system,
                            &mut self.cache,
                            cosmic_text::Color::rgba(0, 0, 0, 255),
                            cosmic_text::Color::rgba(0, 0, 0, 0),
                            cosmic_text::Color::rgba(0, 120, 215, 1),
                            cosmic_text::Color::rgba(255, 255, 255, 255),
                            |x, y, w, h, color| {
                                self.render_context.set_paint(Paint::Solid(PremulRgba8::from_u8_array(color.as_rgba())));
                                self.render_context.fill_rect(
                                    &Rect::new((rect.x + x as f32) as f64, (rect.y + y as f32) as f64, (rect.x + x as f32 + w as f32) as f64, (rect.y + y as f32 + h as f32) as f64)
                                )
                            },
                        );
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };
                }
                RenderCommand::PushLayer(rect) => {

                }
                RenderCommand::PopLayer => {
                }
                RenderCommand::FillBezPath(path, color) => {
                    self.render_context.set_paint(Paint::Solid(color.premultiply().to_rgba8()));
                    self.render_context.fill_path(&path);
                },
                #[cfg(feature = "wgpu_renderer")]
                RenderCommand::FillLyonPath(_, _) => {}
            }
        }
    }

    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>) {
        self.render_context.render_to_pixmap(&mut self.pixmap);
        let buffer = self.copy_pixmap_to_softbuffer(self.pixmap.width as usize, self.pixmap.height as usize);
        buffer.present().expect("Failed to present buffer");
    }
}

impl VelloCpuRenderer {
    fn copy_pixmap_to_softbuffer(&mut self, width: usize, height: usize) -> Buffer<Arc<dyn Window>, Arc<dyn Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();

        let pixmap =  &self.pixmap.buf;

        for offset in 0..(((width * height))) {
            let red = pixmap[4*offset + 0];
            let green = pixmap[4*offset + 1];
            let blue = pixmap[4*offset + 2];
            let alpha = pixmap[4*offset + 3];

            buffer[offset] = rgba_to_encoded_u32(red as u32, green as u32, blue as u32, alpha as u32);
        }

        buffer
    }
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}