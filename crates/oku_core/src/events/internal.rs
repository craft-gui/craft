use crate::components::component::UpdateFn;
use crate::components::ComponentId;
use crate::components::Props;
use crate::events::resource_event::ResourceEvent;
use crate::events::{KeyboardInput, MouseWheel, PointerButton, PointerMoved};
use crate::geometry::Size;
use crate::renderer::renderer::Renderer;
use std::any::Any;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event::Ime;
use winit::window::Window;

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
    ModifiersChanged(winit::event::Modifiers),
    Ime(Ime),
    ProcessUserEvents,
    #[cfg(not(target_arch = "wasm32"))]
    GotUserMessage((UpdateFn, ComponentId, Box<dyn Any + Send + 'static>, Props)),
    #[cfg(target_arch = "wasm32")]
    GotUserMessage((UpdateFn, ComponentId, Box<dyn Any>, Props)),
    ResourceEvent(ResourceEvent),
}
