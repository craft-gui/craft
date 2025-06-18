use crate::components::props::Props;
use crate::elements::element::ElementBoxed;
use crate::events::{CraftMessage, Message};
use crate::reactive::state_store::StateStoreItem;
use crate::GlobalState;

use crate::components::update_result::Event;
use crate::elements::Container;
use crate::window_context::WindowContext;
use std::any::{Any, TypeId};
use std::ops::Deref;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
use winit::event::{Ime, Modifiers};

/// A Component's view function.
pub type ViewFn = fn(
    data: &StateStoreItem,
    global_state: &GlobalState,
    props: Props,
    children: Vec<ComponentSpecification>,
    id: ComponentId,
    window_context: &WindowContext,
) -> ComponentSpecification;

/// A Component's update function.
pub type UpdateFn =
    fn(state: &mut StateStoreItem, global_state: &mut GlobalState, props: Props, event: &mut Event, message: &Message);
pub type ComponentId = u64;

#[derive(Clone, Debug)]
pub struct ComponentData {
    pub default_state: fn() -> Box<StateStoreItem>,
    pub default_props: fn() -> Props,
    pub view_fn: ViewFn,
    pub update_fn: UpdateFn,
    /// A unique identifier for view_fn.
    pub tag: String,
    /// The type id of the view function. This is currently not used.
    pub type_id: TypeId,
}

/// An enum containing either an [`Element`] or a [`ComponentData`].
#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(ComponentData),
    Element(ElementBoxed),
}

/// A specification for components and elements.
#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    /// Specify a key when the component position or type may change, but state should be retained.
    pub key: Option<String>,
    /// A read only reference to the props of the component.
    pub props: Option<Props>,
    /// The children of the component.
    pub children: Vec<ComponentSpecification>,
}

impl ComponentSpecification {
    pub fn new(component: ComponentOrElement) -> Self {
        match component {
            ComponentOrElement::ComponentSpec(component_data) => ComponentSpecification {
                component: ComponentOrElement::ComponentSpec(component_data),
                key: None,
                props: None,
                children: vec![],
            },
            ComponentOrElement::Element(element) => element.into(),
        }
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_owned());
        self
    }

    pub fn props(mut self, props: Props) -> Self {
        self.props = Some(props);
        self
    }

    pub fn push_children(mut self, children: Vec<ComponentSpecification>) -> Self {
        self.children = children;
        self
    }

    pub fn extend_children(mut self, children: Vec<ComponentSpecification>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn push<T>(mut self, component: T) -> Self
    where
        T: Into<ComponentSpecification>,
    {
        self.children.push(component.into());
        self
    }

    pub fn push_in_place(&mut self, component: ComponentSpecification) {
        self.children.push(component);
    }
}

#[macro_export]
macro_rules! component {
    // Match for an associated function or method of a struct
    ($path:path) => {{
        let name = $path;
        ComponentOrElement::ComponentSpec(name, std::any::type_name_of_val(&name).to_string(), name.type_id())
    }};

    // Match for an identifier
    ($name:ident) => {
        ComponentOrElement::ComponentSpec($name, std::any::type_name_of_val(&$name).to_string(), $name.type_id())
    };
}

pub trait Component
where
    Self: 'static + Default + Send,
{
    type GlobalState: 'static + Default + Send;
    type Props: Send + Sync + Default;
    type Message: Any;

    fn generic_view_internal(
        state: &StateStoreItem,
        global_state: &GlobalState,
        props: Props,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
        window_context: &WindowContext,
    ) -> ComponentSpecification {
        let casted_state: &Self = state.downcast_ref::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        if let Some(global_state_casted) = global_state.downcast_ref::<Self::GlobalState>() {
            Self::view(casted_state, global_state_casted, props, children, id, window_context)
        } else {
            #[allow(clippy::unnecessary_mut_passed)]
            Self::view(casted_state, &mut Self::GlobalState::default(), props, children, id, window_context)
        }
    }

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        Container::new().component()
    }

    fn update_internal(
        state: &mut StateStoreItem,
        global_state: &mut GlobalState,
        props: Props,
        event: &mut Event,
        message: &Message,
    ) {
        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        if let Some(global_state_casted) = global_state.downcast_mut::<Self::GlobalState>() {
            Self::update(casted_state, global_state_casted, props, event, message)
        } else {
            Self::update(casted_state, &mut Self::GlobalState::default(), props, event, message)
        }
    }

    fn update(
        &mut self,
        global_state: &mut Self::GlobalState,
        props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        match message {
            Message::CraftMessage(craft_message) => match craft_message {
                CraftMessage::Initialized => {
                    self.on_initialize(global_state, props, event);
                }
                CraftMessage::PointerButtonUp(pointer_message) => {
                    self.on_pointer_button_up(global_state, props, event, pointer_message);
                }
                CraftMessage::PointerButtonDown(pointer_message) => {
                    self.on_pointer_button_down(global_state, props, event, pointer_message);
                }
                CraftMessage::KeyboardInputEvent(keyboard_input) => {
                    self.on_keyboard_input(global_state, props, event, keyboard_input);
                }
                CraftMessage::PointerMovedEvent(pointer_moved) => {
                    self.on_pointer_move(global_state, props, event, pointer_moved);
                }
                CraftMessage::PointerScroll(pointer_scroll_update) => {
                    self.on_pointer_scroll(global_state, props, event, pointer_scroll_update);
                }
                CraftMessage::ImeEvent(ime) => {
                    self.on_ime(global_state, props, event, ime);
                }
                CraftMessage::TextInputChanged(new_string) => {
                    self.on_text_input_changed(global_state, props, event, new_string);
                }
                CraftMessage::LinkClicked(link) => {
                    self.on_link_clicked(global_state, props, event, link);
                }
                CraftMessage::DropdownToggled(dropdown_toggled) => {
                    self.on_dropdown_toggled(global_state, props, event, *dropdown_toggled);
                }
                CraftMessage::DropdownItemSelected(index) => {
                    self.on_dropdown_item_selected(global_state, props, event, *index);
                }
                CraftMessage::SwitchToggled(switch_state) => {
                    self.on_switch_toggled(global_state, props, event, *switch_state);
                }
                CraftMessage::SliderValueChanged(slider_value) => {
                    self.on_slider_value_changed(global_state, props, event, *slider_value);
                }
                CraftMessage::ElementMessage(_) => {}
            },
            crate::events::Message::UserMessage(user_message) => {
                let user_message = user_message.downcast_ref::<Self::Message>();
                if let Some(user_message) = user_message {
                    self.on_user_message(global_state, props, event, user_message);
                }
            }
        }
    }

    fn on_pointer_button_up(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        pointer_event: &PointerButtonUpdate,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_pointer_button_up) = &element.element_data().on_pointer_button_up {
                on_pointer_button_up(self, global_state, event, pointer_event);
            }
        }
    }

    fn on_pointer_button_down(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        pointer_event: &PointerButtonUpdate,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_pointer_button_down) = &element.element_data().on_pointer_button_down {
                on_pointer_button_down(self, global_state, event, pointer_event);
            }
        }
    }

    fn on_initialize(&mut self, global_state: &mut Self::GlobalState, _props: &Self::Props, event: &mut Event) {
        if let Some(element) = event.current_target {
            if let Some(on_initialized) = &element.element_data().on_initialized {
                on_initialized(self, global_state, event);
            }
        }
    }

    fn on_keyboard_input(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        keyboard_input: &KeyboardEvent,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_keyboard_input) = &element.element_data().on_keyboard_input {
                on_keyboard_input(self, global_state, event, keyboard_input);
            }
        }
    }

    fn on_pointer_move(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        pointer_update: &PointerUpdate,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_pointer_move) = &element.element_data().on_pointer_move {
                on_pointer_move(self, global_state, event, pointer_update);
            }
        }
    }

    fn on_user_message(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        _event: &mut Event,
        _user_message: &Self::Message,
    ) {
    }

    fn on_pointer_scroll(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        pointer_scroll_update: &PointerScrollUpdate,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_pointer_scroll) = &element.element_data().on_pointer_scroll {
                on_pointer_scroll(self, global_state, event, pointer_scroll_update);
            }
        }
    }

    fn on_modifiers_changed(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        modifiers: &Modifiers,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_modifiers_changed) = &element.element_data().on_modifiers_changed {
                on_modifiers_changed(self, global_state, event, modifiers);
            }
        }
    }

    fn on_ime(&mut self, global_state: &mut Self::GlobalState, _props: &Self::Props, event: &mut Event, ime: &Ime) {
        if let Some(element) = event.current_target {
            if let Some(on_ime) = &element.element_data().on_ime {
                on_ime(self, global_state, event, ime);
            }
        }
    }

    fn on_text_input_changed(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        new_string: &str,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_text_input_changed) = &element.element_data().on_text_input_changed {
                on_text_input_changed(self, global_state, event, new_string);
            }
        }
    }

    fn on_link_clicked(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        link: &str,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_link_clicked) = &element.element_data().on_link_clicked {
                on_link_clicked(self, global_state, event, link);
            }
        }
    }

    fn on_dropdown_toggled(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        dropdown_toggled: bool,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_dropdown_toggled) = &element.element_data().on_dropdown_toggled {
                on_dropdown_toggled(self, global_state, event, dropdown_toggled);
            }
        }
    }

    fn on_dropdown_item_selected(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        index: usize,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_dropdown_item_selected) = &element.element_data().on_dropdown_item_selected {
                on_dropdown_item_selected(self, global_state, event, index);
            }
        }
    }

    fn on_switch_toggled(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        switch_state: bool,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_switch_toggled) = &element.element_data().on_switch_toggled {
                on_switch_toggled(self, global_state, event, switch_state);
            }
        }
    }

    fn on_slider_value_changed(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        slider_value: f64,
    ) {
        if let Some(element) = event.current_target {
            if let Some(on_slider_value_changed) = &element.element_data().on_slider_value_changed {
                on_slider_value_changed(self, global_state, event, slider_value);
            }
        }
    }

    fn default_state() -> Box<StateStoreItem> {
        Box::<Self>::default()
    }

    fn default_props() -> Props {
        Props::new(Self::Props::default())
    }

    fn component() -> ComponentSpecification {
        let component_data = ComponentData {
            default_state: Self::default_state,
            default_props: Self::default_props,
            view_fn: Self::generic_view_internal,
            update_fn: Self::update_internal,
            tag: std::any::type_name_of_val(&Self::generic_view_internal).to_string(),
            type_id: Self::generic_view_internal.type_id(),
        };

        ComponentSpecification::new(ComponentOrElement::ComponentSpec(component_data))
    }
}
