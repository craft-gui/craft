use crate::components::component::ComponentId;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use parley::FontContext;
use peniko::kurbo;
use tokio::sync::RwLockReadGuard;

#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawRect(Rectangle, Color),
    DrawRectOutline(Rectangle, Color),
    DrawImage(Rectangle, ResourceIdentifier),
    DrawText(Rectangle, ComponentId),
    PushLayer(Rectangle),
    PopLayer,
    FillBezPath(kurbo::BezPath, Color),
}

pub trait Renderer {
    // Surface Functions
    fn surface_width(&self) -> f32;
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn load_font(&mut self, _font_context: &mut FontContext) {}

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color);
    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color);

    fn fill_bez_path(&mut self, path: kurbo::BezPath, color: Color);

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle);
    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier);

    fn push_layer(&mut self, rect: Rectangle);

    fn pop_layer(&mut self);

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_context: &mut FontContext,
        element_state: &mut ElementStateStore,
    );

    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>);
}
