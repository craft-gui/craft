use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, Renderer};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::element_state_store::ElementStateStore;

use tiny_skia::{
    ColorSpace, FillRule, Mask, MaskType, Paint, PathBuilder, Pixmap, PixmapPaint, PixmapRef, Rect, Stroke, Transform,
};

use cosmic_text::{FontSystem, SwashCache};

use image::EncodableLayout;

use peniko::kurbo::BezPath;

use softbuffer::Buffer;

use tokio::sync::RwLockReadGuard;

use winit::window::Window;

use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

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

#[cfg(target_arch = "wasm32")]
unsafe impl Send for Surface {}

pub struct SoftwareRenderer {
    render_commands: Vec<RenderCommand>,

    // Surface
    surface: Surface,
    surface_width: f32,
    surface_height: f32,
    surface_clear_color: Color,
    framebuffer: Vec<(Pixmap, Rectangle)>,
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

        let framebuffer =
            vec![(Pixmap::new(width as u32, height as u32).unwrap(), Rectangle::new(0.0, 0.0, width, height))];

        Self {
            render_commands: vec![],
            surface,
            surface_width: width,
            surface_height: height,
            surface_clear_color: Color::WHITE,
            framebuffer,
            cache: SwashCache::new(),
        }
    }
}

fn draw_rect(canvas: &mut Pixmap, rectangle: Rectangle, fill_color: Color) {
    let mut paint = Paint::default();
    paint.colorspace = ColorSpace::Linear;
    let [r, g, b, a] = fill_color.to_rgba8().to_u8_array();
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(rectangle.x, rectangle.y, rectangle.width, rectangle.height).unwrap();
    canvas.fill_rect(rect, &paint, Transform::identity(), None);
}

fn draw_rect_outline(canvas: &mut Pixmap, rectangle: Rectangle, outline_color: Color) {
    let mut paint = Paint::default();
    paint.colorspace = ColorSpace::Linear;
    let [r, g, b, a] = outline_color.to_rgba8().to_u8_array();
    paint.set_color_rgba8(r, g, b, a);
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
        let width = width.max(1.0);
        let height = height.max(1.0);
        self.surface_width = width;
        self.surface_height = height;
        let framebuffer = Pixmap::new(width as u32, height as u32).unwrap();
        self.surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        self.framebuffer = vec![(framebuffer, Rectangle::new(0.0, 0.0, width, height))];
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
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

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, _rectangle: Rectangle, resource: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(_rectangle, resource));
    }

    fn push_layer(&mut self, rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn prepare(&mut self, resource_manager: RwLockReadGuard<ResourceManager>, font_system: &mut FontSystem, element_state: &ElementStateStore) {
        let framebuffer = self.framebuffer.last_mut().unwrap();
        let framebuffer = &mut framebuffer.0;

        let [r, g, b, a] = self.surface_clear_color.to_rgba8().to_u8_array();
        framebuffer.fill(tiny_skia::Color::from_rgba8(r, g, b, a));

        for command in self.render_commands.drain(..) {
            let framebuffer = self.framebuffer.last_mut().unwrap();
            let framebuffer = &mut framebuffer.0;

            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    draw_rect(framebuffer, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    draw_rect_outline(framebuffer, rectangle, outline_color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        let pixmap = PixmapRef::from_bytes(image.as_bytes(), image.width(), image.height()).unwrap();
                        let pixmap_paint = PixmapPaint::default();
                        framebuffer.draw_pixmap(
                            rectangle.x as i32,
                            rectangle.y as i32,
                            pixmap,
                            &pixmap_paint,
                            Transform::identity(),
                            None,
                        );
                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color) => {
                    let mut paint = Paint::default();
                    paint.colorspace = ColorSpace::Linear;
                    if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextInputState>()
                    {
                        let editor = &text_context.editor;
                        editor.draw(
                            font_system,
                            &mut self.cache,
                            cosmic_text::Color::rgba(0, 0, 0, 255),
                            cosmic_text::Color::rgba(0, 0, 0, 100),
                            cosmic_text::Color::rgba(0, 0, 200, 255),
                            cosmic_text::Color::rgba(255, 255, 255, 255),
                            |x, y, w, h, colora: cosmic_text::Color| {
                                paint.set_color_rgba8(colora.r(), colora.g(), colora.b(), colora.a());
                                framebuffer.fill_rect(
                                    Rect::from_xywh(rect.x + x as f32, rect.y + y as f32, w as f32, h as f32).unwrap(),
                                    &paint,
                                    Transform::identity(),
                                    None,
                                );
                            },
                        );
                    } else if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextState>()
                    {
                        let buffer = &text_context.buffer;

                        let fc = {
                            let [r, g, b, a] = fill_color.to_rgba8().to_u8_array();
                            cosmic_text::Color::rgba(r, g, b, a)
                        };
                        buffer.draw(
                            font_system,
                            &mut self.cache,
                            fc,
                            |x, y, w, h, color| {
                                paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
                                framebuffer.fill_rect(
                                    Rect::from_xywh(rect.x + x as f32, rect.y + y as f32, w as f32, h as f32).unwrap(),
                                    &paint,
                                    Transform::identity(),
                                    None,
                                );
                            },
                        );
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };
                }
                RenderCommand::PushLayer(rect) => {
                    let framebuffer = Pixmap::new(self.surface_width as u32, self.surface_height as u32).unwrap();
                    self.framebuffer.push((framebuffer, rect));
                }
                RenderCommand::PopLayer => {
                    let top_layer = self.framebuffer.pop().unwrap();

                    let clip_rect = top_layer.1;
                    let top_framebuffer = top_layer.0;

                    let mut mask_framebuffer = Pixmap::new(top_framebuffer.width(), top_framebuffer.height()).unwrap();

                    let mut clip_paint = Paint::default();
                    clip_paint.set_color_rgba8(255, 255, 255, 255);

                    mask_framebuffer.fill_rect(
                        Rect::from_ltrb(clip_rect.left(), clip_rect.top(), clip_rect.right(), clip_rect.bottom())
                            .unwrap(),
                        &clip_paint,
                        Transform::identity(),
                        None,
                    );

                    let top_framebuffer = top_framebuffer.as_ref();

                    let current_layer = self.framebuffer.last_mut().unwrap();
                    let current_framebuffer = &mut current_layer.0;

                    let mut paint = Paint::default();
                    paint.colorspace = ColorSpace::Linear;

                    let pixmap_paint = PixmapPaint::default();
                    let mask = Mask::from_pixmap(mask_framebuffer.as_ref(), MaskType::Alpha);

                    current_framebuffer.draw_pixmap(
                        0,
                        0,
                        top_framebuffer,
                        &pixmap_paint,
                        Transform::identity(),
                        Some(&mask),
                    );
                }
                RenderCommand::FillBezPath(_path, _color) => {
                    let mut paint = Paint::default();
                    let [r, g, b, a] = _color.to_rgba8().to_u8_array();
                    paint.set_color_rgba8(r, g, b, a);

                    let mut path = tiny_skia::PathBuilder::new();
                    let mut last_point = (0.0, 0.0);
                    for path_element in _path {
                        match path_element {
                            peniko::kurbo::PathEl::MoveTo(point) => {
                                path.move_to(point.x as f32, point.y as f32);
                                last_point = (point.x as f32, point.y as f32);
                            }
                            peniko::kurbo::PathEl::LineTo(point) => {
                                if last_point.0 == point.x as f32 && last_point.1 == point.y as f32 {
                                    continue;
                                }
                                if last_point.0 == point.x as f32 || last_point.1 == point.y as f32 {
                                    framebuffer.fill_rect(
                                        Rect::from_points(&[
                                            tiny_skia::Point::from_xy(last_point.0, last_point.1),
                                            tiny_skia::Point::from_xy(point.x as f32, point.y as f32),
                                        ])
                                            .unwrap(),
                                        &paint,
                                        Transform::identity(),
                                        None,
                                    );
                                } else {
                                    path.line_to(point.x as f32, point.y as f32);
                                }

                                last_point = (point.x as f32, point.y as f32);
                            }
                            peniko::kurbo::PathEl::QuadTo(point1, point2) => {
                                path.quad_to(point1.x as f32, point1.y as f32, point2.x as f32, point2.y as f32);
                                last_point = (point2.x as f32, point2.y as f32);
                            }
                            peniko::kurbo::PathEl::CurveTo(point1, point2, point3) => {
                                path.cubic_to(
                                    point1.x as f32,
                                    point1.y as f32,
                                    point2.x as f32,
                                    point2.y as f32,
                                    point3.x as f32,
                                    point3.y as f32,
                                );
                                last_point = (point3.x as f32, point3.y as f32);
                            }
                            peniko::kurbo::PathEl::ClosePath => {
                                path.close();
                            }
                        }
                    }
                    let path = path.finish().unwrap();
                    if path.len() <= 2 {
                        continue;
                    }
                    framebuffer.fill_path(&path, &paint, FillRule::EvenOdd, Transform::identity(), None);
                }
            }
        }
    }
    
    fn submit(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>) {
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
                let current_pixel = self.framebuffer.last_mut().unwrap().0.pixels()[index];

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
