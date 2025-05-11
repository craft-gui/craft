use crate::components::props::Props;
use crate::elements::element::ElementBoxed;
use crate::events::{CraftMessage, KeyboardInput, Message, MouseWheel, PointerButton, PointerMoved};
use crate::reactive::state_store::StateStoreItem;
use crate::{GlobalState, WindowContext};

use crate::components::update_result::Event;
use crate::elements::Container;
use std::any::{Any, TypeId};
use std::ops::Deref;
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
#[derive(Clone, Debug)]
pub enum ComponentOrElement {
    ComponentSpec(ComponentData),
    Element(ElementBoxed),
}

/// A specification for components and elements.
#[derive(Clone, Debug)]
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
        _global_state: &mut Self::GlobalState,
        props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        match message {
            Message::CraftMessage(craft_message) => match craft_message {
                CraftMessage::Initialized => {
                    self.on_initialize(props, event);
                }
                CraftMessage::PointerButtonEvent(pointer_button) => {
                    self.on_pointer_button(props, event, pointer_button);
                }
                CraftMessage::KeyboardInputEvent(keyboard_input) => {
                    self.on_keyboard_input(props, event, keyboard_input);
                }
                CraftMessage::PointerMovedEvent(pointer_moved) => {
                    self.on_pointer_move(props, event, pointer_moved);
                }
                CraftMessage::MouseWheelEvent(mouse_wheel) => {
                    self.on_mouse_wheel(props, event, mouse_wheel);
                }
                CraftMessage::ModifiersChangedEvent(modifiers) => {
                    self.on_modifiers_changed(props, event, modifiers);
                }
                CraftMessage::ImeEvent(ime) => {
                    self.on_ime(props, event, ime);
                }
                CraftMessage::TextInputChanged(new_string) => {
                    self.on_text_input_changed(props, event, new_string);
                }
                CraftMessage::DropdownToggled(dropdown_toggled) => {
                    self.on_dropdown_toggled(props, event, *dropdown_toggled);
                }
                CraftMessage::DropdownItemSelected(index) => {
                    self.on_dropdown_item_selected(props, event, *index);
                }
                CraftMessage::SwitchToggled(switch_state) => {
                    self.on_switch_toggled(props, event, *switch_state);
                }
                CraftMessage::SliderValueChanged(slider_value) => {
                    self.on_slider_value_changed(props, event, *slider_value);
                }
            },
            crate::events::Message::UserMessage(user_message) => {
                let user_message = user_message.downcast_ref::<Message>();
                if let Some(user_message) = user_message {
                    self.on_user_message(props, event, user_message);
                }
            }
        }
    }

    fn on_pointer_button(&mut self, _props: &Self::Props, _event: &mut Event, _pointer_button: &PointerButton) {}

    fn on_initialize(&mut self, _props: &Self::Props, _event: &mut Event) {}

    fn on_keyboard_input(&mut self, _props: &Self::Props, _event: &mut Event, _keyboard_input: &KeyboardInput) {}

    fn on_pointer_move(&mut self, _props: &Self::Props, _event: &mut Event, _pointer_moved: &PointerMoved) {}

    fn on_user_message(&mut self, _props: &Self::Props, _event: &mut Event, _user_message: &Message) {}

    fn on_mouse_wheel(&mut self, _props: &Self::Props, _event: &mut Event, _mouse_wheel: &MouseWheel) {}

    fn on_modifiers_changed(&mut self, _props: &Self::Props, _event: &mut Event, _modifiers: &Modifiers) {}

    fn on_ime(&mut self, _props: &Self::Props, _event: &mut Event, _ime: &Ime) {}

    fn on_text_input_changed(&mut self, _props: &Self::Props, _event: &mut Event, _new_string: &str) {}

    fn on_dropdown_toggled(&mut self, _props: &Self::Props, _event: &mut Event, _dropdown_toggled: bool) {}

    fn on_dropdown_item_selected(&mut self, _props: &Self::Props, _event: &mut Event, _index: usize) {}

    fn on_switch_toggled(&mut self, _props: &Self::Props, _event: &mut Event, _switch_state: bool) {}

    fn on_slider_value_changed(&mut self, _props: &Self::Props, _event: &mut Event, _slider_value: f64) {}

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
