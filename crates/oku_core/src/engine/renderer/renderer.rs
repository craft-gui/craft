use crate::components::component::ComponentId;
use crate::engine::renderer::color::Color;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::state_store::StateStore;
use cosmic_text::FontSystem;
use tokio::sync::RwLockReadGuard;

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

pub enum RenderCommand {
    DrawRect(Rectangle, Color),
    DrawRectOutline(Rectangle, Color),
    DrawImage(Rectangle, ResourceIdentifier),
    DrawText(Rectangle, ComponentId, Color),
    PushLayer(Rectangle),
    PopLayer,
}

pub trait Surface {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn present(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

pub trait Renderer {
    // Surface Functions
    fn surface_width(&self) -> f32;
    fn surface_height(&self) -> f32;
    fn present_surface(&mut self);
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color);
    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color);

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color);
    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier);
    
    fn push_layer(&mut self, rect: Rectangle);
    
    fn pop_layer(&mut self);

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &StateStore,
    );
}
