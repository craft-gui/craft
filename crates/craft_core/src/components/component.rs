use crate::components::props::Props;
use crate::elements::element::ElementBoxed;
use crate::events::Event;
use crate::reactive::state_store::StateStoreItem;
use crate::GlobalState;

use crate::components::update_result::UpdateResult;
use crate::elements::Container;
use std::any::{Any, TypeId};
use std::ops::Deref;

/// A Component's view function.
pub type ViewFn = fn(
    data: &StateStoreItem,
    global_state: &GlobalState,
    props: Props,
    children: Vec<ComponentSpecification>,
    id: ComponentId,
) -> ComponentSpecification;

/// A Component's update function.
pub type UpdateFn =
    fn(state: &mut StateStoreItem, global_state: &mut GlobalState, props: Props, message: Event) -> UpdateResult;
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

pub trait Component<T = ()>
where
    Self: 'static + Default + Send,
    T: 'static + Default + Send,
{
    type Props: Send + Sync + Default;

    fn view_with_no_global_state(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new().component()
    }

    fn update_with_no_global_state(_state: &mut Self, _props: &Self::Props, _message: Event) -> UpdateResult {
        UpdateResult::default()
    }

    fn view(
        state: &Self,
        _global_state: &T,
        props: &Self::Props,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        Self::view_with_no_global_state(state, props, children, id)
    }

    fn generic_view(
        state: &StateStoreItem,
        global_state: &GlobalState,
        props: Props,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        let casted_state: &Self = state.downcast_ref::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        if let Some(global_state_casted) = global_state.downcast_ref::<T>() {
            Self::view(casted_state, global_state_casted, props, children, id)
        } else {
            Self::view_with_no_global_state(casted_state, props, children, id)
        }
    }

    fn default_state() -> Box<StateStoreItem> {
        Box::<Self>::default()
    }

    fn default_props() -> Props {
        Props::new(Self::Props::default())
    }

    fn update(state: &mut Self, _global_state: &mut T, props: &Self::Props, message: Event) -> UpdateResult {
        Self::update_with_no_global_state(state, props, message)
    }

    fn generic_update(
        state: &mut StateStoreItem,
        global_state: &mut GlobalState,
        props: Props,
        message: Event,
    ) -> UpdateResult {
        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        if let Some(global_state_casted) = global_state.downcast_mut::<T>() {
            Self::update(casted_state, global_state_casted, props, message)
        } else {
            Self::update_with_no_global_state(casted_state, props, message)
        }
    }

    fn component() -> ComponentSpecification {
        let component_data = ComponentData {
            default_state: Self::default_state,
            default_props: Self::default_props,
            view_fn: Self::generic_view,
            update_fn: Self::generic_update,
            tag: std::any::type_name_of_val(&Self::generic_view).to_string(),
            type_id: Self::generic_view.type_id(),
        };

        ComponentSpecification::new(ComponentOrElement::ComponentSpec(component_data))
    }
}
