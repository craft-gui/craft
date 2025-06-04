use crate::components::component::UpdateFn;
use crate::components::ComponentId;
use crate::components::Props;
use crate::events::resource_event::ResourceEvent;
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use {crate::renderer::renderer::Renderer, std::sync::Arc, winit::window::Window};

pub(crate) struct InternalUserMessage {
    pub update_fn: UpdateFn,
    pub source_component_id: ComponentId,
    #[cfg(not(target_arch = "wasm32"))]
    pub message: Box<dyn Any + Send + Sync + 'static>,
    #[cfg(target_arch = "wasm32")]
    pub message: Box<dyn Any>,
    pub props: Props,
}

pub(crate) enum InternalMessage {
    GotUserMessage(InternalUserMessage),
    ResourceEvent(ResourceEvent),
    #[cfg(target_arch = "wasm32")]
    RendererCreated(Arc<Window>, Box<dyn Renderer>),
}
