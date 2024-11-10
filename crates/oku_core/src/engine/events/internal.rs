use crate::engine::events::resource_event::ResourceEvent;
use crate::engine::events::{PointerButton, PointerMoved, KeyboardInput, MouseWheel};
use crate::engine::renderer::renderer::Renderer;
use crate::components::component::UpdateFn;
use std::any::Any;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub enum InternalMessage {
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
    GotUserMessage((UpdateFn, u64, Option<String>, Box<dyn Any + Send>)),
    ResourceEvent(ResourceEvent),
}
