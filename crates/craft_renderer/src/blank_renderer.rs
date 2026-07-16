use std::any::Any;
use std::sync::Arc;

use craft_primitives::Color;
use craft_primitives::geometry::Rectangle;
use craft_resource_manager::ResourceManager;

use crate::render_list::RenderList;
use crate::renderer::Renderer;

#[derive(Default)]
pub struct BlankRenderer {
    render_list: RenderList
}

impl Renderer for BlankRenderer {
    fn surface_width(&self) -> f32 {
        0.0
    }

    fn surface_height(&self) -> f32 {
        0.0
    }

    fn resize_surface(&mut self, _width: f32, _height: f32) {}

    fn surface_set_clear_color(&mut self, _color: Color) {}

    fn render_list(&self) -> &RenderList {
        &self.render_list
    }

    fn render_list_mut(&mut self) -> &mut RenderList {
        &mut self.render_list
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare<'a>(
        &mut self,
        _resource_manager: Arc<ResourceManager>,
        _window: Rectangle,
    ) {
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {}
}
