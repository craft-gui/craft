use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::renderer::{Renderer, TextScroll};
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo::BezPath;
use tokio::sync::RwLockReadGuard;
use crate::renderer::Brush;
use crate::renderer::text::BufferGlyphs;

pub struct BlankRenderer;

impl Renderer for BlankRenderer {
    fn surface_width(&self) -> f32 {
        0.0
    }

    fn surface_height(&self) -> f32 {
        0.0
    }

    fn resize_surface(&mut self, _width: f32, _height: f32) {}

    fn surface_set_clear_color(&mut self, _color: Color) {}

    fn draw_rect(&mut self, _rectangle: Rectangle, _fill_color: Color) {}

    fn draw_rect_outline(&mut self, _rectangle: Rectangle, _outline_color: Color) {}

    fn fill_bez_path(&mut self, _path: BezPath, _brush: Brush) {}

    fn draw_text(
        &mut self,
        _buffer_glyphs: BufferGlyphs,
        _rectangle: Rectangle,
        _text_scroll: Option<TextScroll>,
        _show_cursor: bool,
    ) {
    }

    fn draw_image(&mut self, _rectangle: Rectangle, _resource_identifier: ResourceIdentifier) {}

    fn push_layer(&mut self, _rect: Rectangle) {}

    fn pop_layer(&mut self) {}

    fn prepare(
        &mut self,
        _resource_manager: RwLockReadGuard<ResourceManager>,
        _font_system: &mut FontSystem,
    ) {
    }

    fn submit(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>) {}
}
