use crate::components::component::UpdateFn;
use crate::components::props::Props;
use crate::engine::events::resource_event::ResourceEvent;
use crate::engine::events::{KeyboardInput, MouseWheel, OkuEvent, PointerButton, PointerMoved};
use crate::engine::renderer::renderer::Renderer;
use std::any::Any;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub(crate) enum InternalMessage {
    RequestRedraw,
    Close,
    Confirmation,
    Resume(Arc<dyn Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    PointerButton(PointerButton),
    PointerMoved(PointerMoved),
    MouseWheel(MouseWheel),
    KeyboardInput(KeyboardInput),
    ProcessUserEvents,
    GotUserMessage((UpdateFn, u64, Option<String>, Box<dyn Any + Send>, Option<Props>)),
    ResourceEvent(ResourceEvent),
    ElementEvent(OkuEvent),
}
