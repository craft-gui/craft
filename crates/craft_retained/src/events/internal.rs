use craft_resource_manager::resource_event::ResourceEvent;
#[cfg(target_arch = "wasm32")]
use {craft_renderer::renderer::Renderer, std::sync::Arc, winit::window::Window};

pub enum InternalMessage {
    ResourceEvent(ResourceEvent),
    #[cfg(target_arch = "wasm32")]
    RendererCreated(Arc<Window>, Box<dyn Renderer>),
}

impl From<ResourceEvent> for InternalMessage {
    fn from(event: ResourceEvent) -> Self {
        InternalMessage::ResourceEvent(event)
    }
}
