use crate::components::component::UpdateFn;
use crate::components::props::Props;
use crate::events::resource_event::ResourceEvent;
use crate::events::{KeyboardInput, MouseWheel, PointerButton, PointerMoved};
use crate::renderer::renderer::Renderer;
use std::any::Any;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::components::ComponentId;
use crate::geometry::Size;

pub(crate) enum InternalMessage {
    RequestRedraw(f64, Size),
    Close,
    Confirmation,
    Resume(Arc<dyn Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    PointerButton(PointerButton),
    PointerMoved(PointerMoved),
    MouseWheel(MouseWheel),
    KeyboardInput(KeyboardInput),
    ProcessUserEvents,
    #[cfg(not(target_arch = "wasm32"))]
    GotUserMessage((UpdateFn, ComponentId, Box<dyn Any + Send>, Props)),
    #[cfg(target_arch = "wasm32")]
    GotUserMessage((UpdateFn, ComponentId, Box<dyn Any>, Props)),
    ResourceEvent(ResourceEvent),
}
