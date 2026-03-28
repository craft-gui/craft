use std::any::Any;
use std::sync::Arc;

use craft_primitives::geometry::Rectangle;
use craft_primitives::Color;

use craft_resource_manager::ResourceManager;

use crate::render_list::RenderList;
pub use crate::screenshot::Screenshot;
use crate::sort_commands::sort_and_cull_render_list_internal;

pub trait Renderer: Any {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn sort_and_cull_render_list(&mut self, render_list: &mut RenderList) {
        sort_and_cull_render_list_internal(self.surface_height(), render_list);
    }
    fn prepare_render_list<'a>(
        &'a mut self,
        render_list: &'a mut RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    );

    fn submit(&mut self, resource_manager: Arc<ResourceManager>);

    fn screenshot(&self) -> Screenshot {
        Screenshot {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }
}