use crate::components::props::Props;
use crate::elements::element::ElementBoxed;
use crate::events::{CraftMessage, Message};
use crate::reactive::state_store::StateStoreItem;
use crate::GlobalState;

use crate::components::update_result::Event;
use crate::elements::{Container, Element};
use crate::window_context::WindowContext;
use std::any::{Any, TypeId};
use std::ops::Deref;

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
pub type UpdateFn = fn(
    state: &mut StateStoreItem,
    global_state: &mut GlobalState,
    props: Props,
    event: &mut Event,
    message: &Message,
    id: ComponentId,
    window_context: &mut WindowContext,
    target: Option<&dyn Element>,
    current_target: Option<&dyn Element>,
);

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

pub struct Context<'a, C: Component> {
    component: Option<&'a C>,
    component_mut: Option<&'a mut C>,
    global_state: Option<&'a GlobalState>,
    global_state_mut: Option<&'a mut GlobalState>,
    props: Props,
    children: Option<Vec<ComponentSpecification>>,
    id: ComponentId,
    window: Option<&'a WindowContext>,
    window_mut: Option<&'a mut WindowContext>,
    message: Option<&'a Message>,
    event: Option<&'a mut Event>,
    target: Option<&'a dyn Element>,
    current_target: Option<&'a dyn Element>,
}

impl<'a, ComponentType: Component> Context<'a, ComponentType> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        component: Option<&'a ComponentType>,
        component_mut: Option<&'a mut ComponentType>,
        global_state: Option<&'a GlobalState>,
        global_state_mut: Option<&'a mut GlobalState>,
        props: Props,
        children: Option<Vec<ComponentSpecification>>,
        id: ComponentId,
        window: Option<&'a WindowContext>,
        window_mut: Option<&'a mut WindowContext>,
        event: Option<&'a mut Event>,
        message: Option<&'a Message>,
        target: Option<&'a dyn Element>,
        current_target: Option<&'a dyn Element>,
    ) -> Self {
        Context {
            component,
            component_mut,
            global_state,
            global_state_mut,
            props,
            children,
            id,
            window,
            window_mut,
            event,
            message,
            target,
            current_target,
        }
    }

    pub fn props(&self) -> &ComponentType::Props {
        let props: &ComponentType::Props = self.props.data.deref().downcast_ref().unwrap();
        props
    }

    pub fn state_mut(&mut self) -> &mut ComponentType {
        self.component_mut.as_deref_mut().unwrap()
    }

    pub fn state(&self) -> &ComponentType {
        if self.component_mut.is_some() {
            self.component_mut.as_deref().unwrap()
        } else {
            self.component.as_ref().unwrap()
        }
    }

    pub fn global_state(& self) -> &ComponentType::GlobalState {
        self.global_state.unwrap().downcast_ref::<ComponentType::GlobalState>().expect("Global state type mismatch")
    }

    pub fn global_state_mut(&mut self) -> &mut ComponentType::GlobalState {
        self.global_state_mut
            .as_deref_mut()
            .expect("Global ")
            .downcast_mut::<ComponentType::GlobalState>()
            .expect("Global state type mismatch")
    }

    pub fn message(&self) -> &Message {
        self.message.as_ref().expect("Message is not set")
    }

    pub fn event_mut(&mut self) -> &mut Event {
        self.event.as_deref_mut().expect("Event is not set")
    }

    pub fn current_target(&self) -> Option<&dyn Element> {
        self.current_target
    }

    pub fn target(&self) -> Option<&dyn Element> {
        self.target
    }

    pub fn children(&self) -> &Vec<ComponentSpecification> {
        self.children.as_ref().expect("Children are not set")
    }

    pub fn children_mut(&mut self) -> &mut Vec<ComponentSpecification> {
        self.children.as_mut().expect("Children are not set")
    }

    pub fn window(&self) -> &WindowContext {
        self.window.unwrap()
    }

    pub fn window_mut(&mut self) -> &mut WindowContext {
        self.window_mut.as_deref_mut().unwrap()
    }
    
    pub fn id(&self) -> ComponentId {
        self.id
    }
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_event<ComponentType: Component>(
    state: &mut StateStoreItem,
    global_state: &mut GlobalState,
    props: Props,
    event: &mut Event,
    message: &Message,
    id: ComponentId,
    window_context: &mut WindowContext,
    target: Option<&dyn Element>,
    current_target: Option<&dyn Element>,
) {
    match message {
        Message::CraftMessage(craft_message) => match craft_message {
            CraftMessage::Initialized => {}
            CraftMessage::PointerButtonUp(pointer_button_update) => {
                if let Some(element) = current_target {
                    if let Some(on_pointer_button_up) = &element.element_data().event_handlers.on_pointer_up {
                        on_pointer_button_up(
                            state,
                            global_state,
                            props,
                            event,
                            message,
                            id,
                            window_context,
                            target,
                            current_target,
                            pointer_button_update,
                        );
                    }
                }
            }
            CraftMessage::PointerButtonDown(_) => {}
            CraftMessage::KeyboardInputEvent(_) => {}
            CraftMessage::PointerMovedEvent(_) => {}
            CraftMessage::PointerScroll(_) => {}
            CraftMessage::ImeEvent(_) => {}
            CraftMessage::TextInputChanged(_) => {}
            CraftMessage::LinkClicked(_) => {}
            CraftMessage::DropdownToggled(_) => {}
            CraftMessage::DropdownItemSelected(_) => {}
            CraftMessage::SwitchToggled(_) => {}
            CraftMessage::SliderValueChanged(_) => {}
            CraftMessage::ElementMessage(_) => {}
        },
        Message::UserMessage(user_message) => {
            let casted_state: &mut ComponentType = state.downcast_mut::<ComponentType>().unwrap();
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
                None,
                None,
                None,
                None,
            );
            let user_message: Option<&ComponentType::Message> = user_message.as_any().downcast_ref();
            if let Some(user_message) = user_message {
                ComponentType::on_user_message(&mut context, user_message);
            }
        }
    }
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

        let mut context = Context::new(
            Some(casted_state),
            None,
            Some(global_state),
            None,
            props,
            Some(children),
            id,
            Some(window_context),
            None,
            None,
            None,
            None,
            None,
        );
        Self::view(&mut context)
    }

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {
        Container::new().component()
    }


    #[allow(clippy::too_many_arguments)]
    fn update_internal(
        state: &mut StateStoreItem,
        global_state: &mut GlobalState,
        props: Props,
        event: &mut Event,
        message: &Message,
        id: ComponentId,
        window_context: &mut WindowContext,
        target: Option<&dyn Element>,
        current_target: Option<&dyn Element>,
    ) {
        dispatch_event::<Self>(
            state,
            global_state,
            props.clone(),
            event,
            message,
            id,
            window_context,
            target,
            current_target,
        );

        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();

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
        Self::update(&mut context);
    }

    fn update(_context: &mut Context<Self>) {}

    fn on_user_message(_context: &mut Context<Self>, _message: &Self::Message) {}

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
