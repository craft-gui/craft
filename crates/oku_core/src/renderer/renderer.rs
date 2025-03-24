use crate::components::component::ComponentId;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo;
use tokio::sync::RwLockReadGuard;

#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawRect(Rectangle, Color),
    DrawRectOutline(Rectangle, Color),
    DrawImage(Rectangle, ResourceIdentifier),
    DrawText(Rectangle, ComponentId, Color, Option<TextScroll>),
    PushLayer(Rectangle),
    PopLayer,
    FillBezPath(kurbo::BezPath, Color),
    #[cfg(feature = "wgpu_renderer")]
    FillLyonPath(lyon::path::Path, Color),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextScroll {
    pub scroll_y: f32,
    pub scroll_height: f32,
}

impl TextScroll {
    pub fn new(scroll_y: f32, scroll_height: f32) -> Self {
        Self {
            scroll_y,
            scroll_height,
        }
    }
}

pub trait Renderer {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn load_font(&mut self, _font_system: &mut FontSystem) {

    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color);
    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color);

    fn fill_bez_path(&mut self, path: kurbo::BezPath, color: Color);
    #[allow(dead_code)]
    #[cfg(feature = "wgpu_renderer")]
    fn fill_lyon_path(&mut self, path: &lyon::path::Path, color: Color);

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color, text_scroll: Option<TextScroll>);
    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier);

    fn push_layer(&mut self, rect: Rectangle);

    fn pop_layer(&mut self);

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &ElementStateStore,
    );

    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>);
}
