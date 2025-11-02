use std::any::Any;
use craft_primitives::geometry::Rectangle;
use craft_primitives::Color;
use crate::renderer::{RenderList, Renderer};
use craft_resource_manager::ResourceManager;
use std::sync::Arc;
use crate::text_renderer_data::TextRender;

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
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn prepare_render_list<'a>(
        &mut self,
        _render_list: &mut RenderList,
        _resource_manager: Arc<ResourceManager>,
        _window: Rectangle,
    ) {

    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {}
}
