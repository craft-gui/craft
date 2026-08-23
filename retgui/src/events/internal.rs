#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use retgui_resource_manager::resource_event::ResourceEvent;
#[cfg(target_arch = "wasm32")]
use {retgui_renderer::renderer::Renderer, std::sync::Arc, winit::window::Window};

pub enum InternalMessage {
    ResourceEvent(ResourceEvent),
    #[cfg(target_arch = "wasm32")]
    RendererCreated(Arc<dyn Window>, Rc<RefCell<dyn Renderer>>),
}

impl From<ResourceEvent> for InternalMessage {
    fn from(event: ResourceEvent) -> Self {
        InternalMessage::ResourceEvent(event)
    }
}
