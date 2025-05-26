use crate::components::component::UpdateFn;
use crate::components::ComponentId;
use crate::components::Props;
use crate::events::resource_event::ResourceEvent;
use crate::geometry::Size;
use crate::renderer::renderer::Renderer;
use std::any::Any;
use std::sync::Arc;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
use winit::dpi::PhysicalSize;
use winit::event::Ime;
use winit::window::Window;
use crate::App;

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
    RequestRedraw(f64, Size<f32>),
    Close,
    Confirmation,
    Resume(Arc<Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    PointerButtonUp(PointerButtonUpdate),
    PointerButtonDown(PointerButtonUpdate),
    PointerMoved(PointerUpdate),
    PointerScroll(PointerScrollUpdate),
    KeyboardInput(KeyboardEvent),
    Ime(Ime),
    ProcessUserEvents,
    GotUserMessage(InternalUserMessage),
    ResourceEvent(ResourceEvent),
    TakeApp(Box<App>),
}
