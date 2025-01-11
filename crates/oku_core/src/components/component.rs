use crate::components::props::Props;
use crate::elements::element::{Element, ElementBox};
use crate::events::{Event, OkuMessage};
use crate::reactive::state_store::StateStoreItem;
use crate::{FutureAny, PinnedFutureAny};
use std::any::{Any, TypeId};
use std::future::Future;
use std::ops::Deref;

/// A Component's view function.
pub type ViewFn =
    fn(data: &StateStoreItem, props: Props, children: Vec<ComponentSpecification>) -> ComponentSpecification;

/// The result of an update.
pub struct UpdateResult {
    /// Propagate oku_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an oku_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
    pub(crate) result_message: Option<OkuMessage>,
}

impl UpdateResult {

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_result<T: Send + 'static>(t: T) -> Box<dyn Any + Send + 'static> {
        Box::new(t)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_result<T: 'static>(t: T) -> Box<dyn Any + 'static> {
        Box::new(t)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_no_result() -> Box<dyn Any + Send + 'static> {
        Box::new(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_no_result() -> Box<dyn Any + 'static> {
        Box::new(())
    }

}

impl Default for UpdateResult {
    fn default() -> Self {
        UpdateResult {
            propagate: true,
            future: None,
            prevent_defaults: false,
            result_message: None,
        }
    }
}

impl UpdateResult {
    pub fn new() -> UpdateResult {
        UpdateResult::default()
    }

    pub fn pinned_future(mut self, future: PinnedFutureAny) -> Self {
        self.future = Some(future);
        self
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn future<F: Future<Output = Box<dyn Any + Send>> + 'static + Send>(mut self, future: F) -> Self {
        self.future = Some(Box::pin(future));
        self
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn future<F: Future<Output = Box<dyn Any>> + 'static>(mut self, future: F) -> Self {
        self.future = Some(Box::pin(future));
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

    pub(crate) fn result_message(mut self, message: OkuMessage) -> Self {
        self.result_message = Some(message);
        self
    }
}

/// A Component's update function.
pub type UpdateFn = fn(state: &mut StateStoreItem, props: Props, message: Event) -> UpdateResult;
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
    Element(ElementBox),
}

/// A specification for components and elements.
#[derive(Clone, Debug)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentSpecification>,
}

impl ComponentSpecification {
    pub fn new(component: ComponentOrElement) -> Self {
        match component {
            ComponentOrElement::ComponentSpec(component_data) => {
                ComponentSpecification {
                    component: ComponentOrElement::ComponentSpec(component_data),
                    key: None,
                    props: None,
                    children: vec![],
                }
            }
            ComponentOrElement::Element(element) => {
                element.into()
            }
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

    pub fn push<T>(mut self, component: T) -> Self
    where
        T: Into<ComponentSpecification>,
    {
        self.children.push(component.into());
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
    type Props: Send + Sync + Default;

    fn view(state: &Self, props: &Self::Props, children: Vec<ComponentSpecification>) -> ComponentSpecification;

    fn generic_view(
        state: &StateStoreItem,
        props: Props,
        children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        let casted_state: &Self = state.downcast_ref::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        Self::view(casted_state, props, children)
    }

    fn default_state() -> Box<StateStoreItem> {
        Box::<Self>::default()
    }

    fn default_props() -> Props {
        Props::new(Self::Props::default())
    }

    fn update(_state: &mut Self, _props: &Self::Props, _message: Event) -> UpdateResult {
        UpdateResult::new()
    }

    fn generic_update(state: &mut StateStoreItem, props: Props, message: Event) -> UpdateResult {
        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();
        let props: &Self::Props = props.data.deref().downcast_ref().unwrap();

        Self::update(casted_state, props, message)
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