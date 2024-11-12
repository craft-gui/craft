use crate::engine::events::{Message, OkuEvent};
use crate::components::props::Props;
use crate::elements::element::Element;
use crate::PinnedFutureAny;
use std::any::{Any, TypeId};
use std::ops::Deref;
use crate::reactive::state_store::StateStoreItem;

/// A Component's view function.
pub type ViewFn = fn(
    data: &StateStoreItem,
    props: Option<Props>,
    children: Vec<ComponentSpecification>,
    id: ComponentId,
) -> ComponentSpecification;

/// The result of an update.
pub struct UpdateResult {
    /// Propagate oku_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an oku_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
    pub(crate) result_message: Option<OkuEvent>
}

impl Default for UpdateResult {
    fn default() -> Self {
        UpdateResult {
            propagate: true,
            future: None,
            prevent_defaults: false,
            result_message: None
        }
    }
}

impl UpdateResult {
    pub fn new() -> UpdateResult {
        UpdateResult::default()
    }

    pub fn future(mut self, future: PinnedFutureAny) -> Self {
        self.future = Some(future);
        self
    }

    pub fn prevent_defaults(mut self) -> Self {
        self.prevent_defaults = true;
        self
    }

    pub fn prevent_propagate(mut self) -> Self {
        self.propagate = false;
        self
    }

    pub(crate) fn result_message(mut self, message: OkuEvent) -> Self {
        self.result_message = Some(message);
        self
    }
}

/// A Component's update function.
pub type UpdateFn = fn(
    state: &mut StateStoreItem,
    props: Option<Props>,
    id: ComponentId,
    message: Message,
    source_element_id: Option<String>,
) -> UpdateResult;
pub type ComponentId = u64;

#[derive(Clone)]
pub struct ComponentData {
    pub default_state: fn() -> Box<StateStoreItem>,
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
    Element(Box<dyn Element>),
}

/// A specification for components and elements.
#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentSpecification>,
}

impl ComponentSpecification {
    pub fn new(component: ComponentOrElement) -> Self {
        ComponentSpecification {
            component,
            key: None,
            props: None,
            children: vec![],
        }
    }

    pub fn key(mut self, key: &str) -> Self {
        if let ComponentOrElement::Element(_) = self.component { 
            panic!("Component cannot have a key.")
        }
        self.key = Some(key.to_owned());
        self
    }

    pub fn props(mut self, props: Props) -> Self {
        self.props = Some(props);
        self
    }

    pub fn children(mut self, children: Vec<ComponentSpecification>) -> Self {
        self.children = children;
        self
    }

    pub fn push(mut self, component: ComponentSpecification) -> Self {
        self.children.push(component);
        self
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
    type Props: Send + Sync;

    fn view(
        state: &Self,
        props: Option<&Self::Props>,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification;

    fn generic_view(
        state: &StateStoreItem,
        props: Option<Props>,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        let casted_state: &Self = state.downcast_ref::<Self>().unwrap();
        let props: Option<&Self::Props> = props.as_ref().map(|props| props.data.deref().downcast_ref().unwrap());

        Self::view(casted_state, props, children, id)
    }

    fn default_state() -> Box<StateStoreItem> {
        Box::<Self>::default()
    }

    fn update(state: &mut Self, props: Option<&Self::Props>, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        UpdateResult::new()
    }

    fn generic_update(
        state: &mut StateStoreItem,
        props: Option<Props>,
        id: ComponentId,
        message: Message,
        source_element: Option<String>,
    ) -> UpdateResult {
        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();
        let props: Option<&Self::Props> = props.as_ref().map(|props| props.data.deref().downcast_ref().unwrap());

        Self::update(casted_state, props, id, message, source_element)
    }

    fn component() -> ComponentSpecification {
        let component_data = ComponentData {
            default_state: Self::default_state,
            view_fn: Self::generic_view,
            update_fn: Self::generic_update,
            tag: std::any::type_name_of_val(&Self::generic_view).to_string(),
            type_id: Self::generic_view.type_id(),
        };

        ComponentSpecification::new(ComponentOrElement::ComponentSpec(component_data))
    }
}
