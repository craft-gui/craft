use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand, Renderer};
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::components::component::{ComponentId, GenericUserState};
use cosmic_text::{BufferRef, Edit, FontSystem, SwashCache};
use image::EncodableLayout;
use log::info;
use softbuffer::Buffer;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tiny_skia::{ColorSpace, Paint, PathBuilder, Pixmap, PixmapPaint, PixmapRef, Rect, Stroke, Transform};
use tokio::sync::RwLockReadGuard;
use winit::window::Window;
use crate::elements::element::ElementState;
use crate::platform::resource_manager::resource::Resource;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;

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
impl Deref for Surface{
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

#[cfg(target_arch = "wasm32")]
unsafe impl Send for Surface {}

pub struct SoftwareRenderer {
    render_commands: Vec<RenderCommand>,

    // Surface
    surface: Surface,
    surface_width: f32,
    surface_height: f32,
    surface_clear_color: Color,
    framebuffer: Pixmap,
    cache: SwashCache,
}


impl SoftwareRenderer {
    pub(crate) fn new(window: Arc<dyn Window>) -> Self {
        let width = window.surface_size().width.max(1) as f32;
        let height = window.surface_size().height.max(1) as f32;
        
        let mut surface = Surface::new(window.clone());
        surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        
        let framebuffer = Pixmap::new(width as u32, height as u32).unwrap();

        Self {
            render_commands: vec![],
            surface,
            surface_width: width,
            surface_height: height,
            surface_clear_color: Color::new_from_rgba_u8(255, 255, 255, 255),
            framebuffer,
            cache: SwashCache::new(),
        }
    }
}

fn draw_rect(canvas: &mut Pixmap, rectangle: Rectangle, fill_color: Color) {
    let mut paint = Paint::default();
    paint.colorspace = ColorSpace::Linear;
    paint.set_color_rgba8(fill_color.r_u8(), fill_color.g_u8(), fill_color.b_u8(), fill_color.a_u8());
    paint.anti_alias = true;

    let rect = Rect::from_xywh(rectangle.x, rectangle.y, rectangle.width, rectangle.height).unwrap();
    canvas.fill_rect(rect, &paint, Transform::identity(), None);
}

fn draw_rect_outline(canvas: &mut Pixmap, rectangle: Rectangle, outline_color: Color) {
    let mut paint = Paint::default();
    paint.colorspace = ColorSpace::Linear;
    paint.set_color_rgba8(outline_color.r_u8(), outline_color.g_u8(), outline_color.b_u8(), outline_color.a_u8());
    paint.anti_alias = true;

    let rect = Rect::from_xywh(rectangle.x, rectangle.y, rectangle.width, rectangle.height).unwrap();

    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    let path = pb.finish().unwrap();

    // Set up the stroke
    let stroke = Stroke {
        width: 2.0, // Stroke width
        ..Stroke::default()
    };
    canvas.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}

impl Renderer for SoftwareRenderer {
    fn surface_width(&self) -> f32 {
        self.surface_width
    }

    fn surface_height(&self) -> f32 {
        self.surface_height
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.surface_width = width;
        self.surface_height = height;
        let framebuffer = Pixmap::new(width as u32, height as u32).unwrap();
        self.surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        self.framebuffer = framebuffer;
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
        self.render_commands.push(RenderCommand::DrawRectOutline(rectangle, Color::RED));
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        self.render_commands.push(RenderCommand::DrawRectOutline(rectangle, outline_color));
    }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, _rectangle: Rectangle, resource: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(_rectangle, resource));
        info!("Image added");
    }

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &HashMap<ComponentId, Box<ElementState>>,
    ) {
        self.framebuffer.fill(tiny_skia::Color::from_rgba8(
            self.surface_clear_color.r_u8(),
            self.surface_clear_color.g_u8(),
            self.surface_clear_color.b_u8(),
            self.surface_clear_color.a_u8(),
        ));

        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    draw_rect(&mut self.framebuffer, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    draw_rect_outline(&mut self.framebuffer, rectangle, outline_color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        let pixmap = PixmapRef::from_bytes(image.as_bytes(), image.width(), image.height()).unwrap();
                        let pixmap_paint = PixmapPaint::default();
                        self.framebuffer.draw_pixmap(rectangle.x as i32, rectangle.y as i32, pixmap, &pixmap_paint, Transform::identity(), None);
                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color) => {
                    let buffer = if let Some(text_context) = element_state.get(&component_id).unwrap().downcast_ref::<TextInputState>() {
                        match text_context.editor.buffer_ref() {
                            BufferRef::Owned(buffer) => buffer,
                            BufferRef::Borrowed(_) => panic!("Editor must own buffer."),
                            BufferRef::Arc(_) =>  panic!("Editor must own buffer.")
                        }
                    } else if let Some(text_context) = element_state.get(&component_id).unwrap().downcast_ref::<TextState>() {
                        &text_context.buffer
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };

                    let mut paint = Paint::default();
                    paint.colorspace = ColorSpace::Linear;
                    buffer.draw(
                        font_system,
                        &mut self.cache,
                        cosmic_text::Color::rgba(
                            fill_color.r_u8(),
                            fill_color.g_u8(),
                            fill_color.b_u8(),
                            fill_color.a_u8(),
                        ),
                        |x, y, w, h, color| {
                            paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
                            self.framebuffer.fill_rect(
                                Rect::from_xywh(rect.x + x as f32, rect.y + y as f32, w as f32, h as f32).unwrap(),
                                &paint,
                                Transform::identity(),
                                None,
                            );
                        },
                    );
                }
            }
        }
        let buffer = self.copy_skia_buffer_to_softbuffer(self.surface_width, self.surface_height);
        buffer.present().unwrap();
    }
}

impl SoftwareRenderer {
    fn copy_skia_buffer_to_softbuffer(&mut self, width: f32, height: f32) -> Buffer<Arc<dyn Window>, Arc<dyn Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let index = y as usize * width as usize + x as usize;
                let current_pixel = self.framebuffer.pixels()[index];

                let red = current_pixel.red() as u32;
                let green = current_pixel.green() as u32;
                let blue = current_pixel.blue() as u32;
                let alpha = current_pixel.alpha() as u32;

                buffer[index] = rgba_to_encoded_u32(red, green, blue, alpha);
            }
        }
        buffer
    }
}
