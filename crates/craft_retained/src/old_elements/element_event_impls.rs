// This file is generated via build.rs. Do not modify manually!

use crate::components::Component;
use crate::components::Context;
use std::sync::Arc;

use crate::components::ComponentId;
use crate::components::Event;
use crate::components::Props;
use crate::elements::Element;
use crate::events::Message;
use crate::reactive::state_store::StateStoreItem;
use crate::{GlobalState, WindowContext};

use crate::elements::Container;
use crate::elements::Text;
use crate::elements::TextInput;
use ui_events::pointer::PointerButtonUpdate;

impl Container {
    pub fn on_pointer_up<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_up = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl TextInput {
    pub fn on_pointer_up<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_up = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl Text {
    pub fn on_pointer_up<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_up = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl Container {
    pub fn on_pointer_down<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_down = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl TextInput {
    pub fn on_pointer_down<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_down = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl Text {
    pub fn on_pointer_down<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &PointerButtonUpdate) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_pointer_down = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &PointerButtonUpdate| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}

impl TextInput {
    pub fn on_link_clicked<ComponentType: Component>(
        mut self,
        callback: impl Fn(&mut Context<ComponentType>, &str) + 'static + Send + Sync,
    ) -> Self {
        self.element_data_mut().event_handlers.on_link_clicked = Some(Arc::new(
            move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,
                  press: &str| {
                if let Some(casted_state) = state.downcast_mut::<ComponentType>() {
                    let mut context = Context::new(
                        None,
                        Some(casted_state),
                        None,
                        Some(global_state),
                        props,
                        None,
                        id,
                        None,
                        Some(window_context),
                        Some(event),
                        Some(message),
                        target,
                        current_target,
                    );
                    callback(&mut context, press);
                } else {
                    panic!("Invalid type passed to callback.");
                }
            },
        ));
        self
    }
}
