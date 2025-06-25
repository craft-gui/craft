// This file is generated via build.rs. Do not modify manually!

use std::sync::Arc;

use crate::components::Event;
use crate::elements::Element;
use crate::events::Message;
use crate::reactive::state_store::StateStoreItem;
use crate::{GlobalState, WindowContext};

use ui_events::pointer::PointerButtonUpdate;
#[derive(Clone, Default)]
pub struct EventHandlers {
    pub(crate) on_pointer_up: Option<
        Arc<
            dyn Fn(
                    &mut StateStoreItem,
                    &mut GlobalState,
                    crate::components::Props,
                    &mut Event,
                    &Message,
                    crate::components::component::ComponentId,
                    &mut WindowContext,
                    Option<&dyn Element>,
                    Option<&dyn Element>,
                    &PointerButtonUpdate,
                ) + Send
                + Sync,
        >,
    >,
    pub(crate) on_pointer_down: Option<
        Arc<
            dyn Fn(
                    &mut StateStoreItem,
                    &mut GlobalState,
                    crate::components::Props,
                    &mut Event,
                    &Message,
                    crate::components::component::ComponentId,
                    &mut WindowContext,
                    Option<&dyn Element>,
                    Option<&dyn Element>,
                    &PointerButtonUpdate,
                ) + Send
                + Sync,
        >,
    >,
    pub(crate) on_link_clicked: Option<
        Arc<
            dyn Fn(
                    &mut StateStoreItem,
                    &mut GlobalState,
                    crate::components::Props,
                    &mut Event,
                    &Message,
                    crate::components::component::ComponentId,
                    &mut WindowContext,
                    Option<&dyn Element>,
                    Option<&dyn Element>,
                    &str,
                ) + Send
                + Sync,
        >,
    >,
}
