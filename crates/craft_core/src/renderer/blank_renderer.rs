use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderList, Renderer};
use crate::resource_manager::ResourceManager;
use cosmic_text::FontSystem;
use std::sync::Arc;

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

    fn prepare_render_list(
        &mut self,
        _render_list: RenderList,
        _resource_manager: Arc<ResourceManager>,
        _font_system: &mut FontSystem,
    ) {}

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {}
}
