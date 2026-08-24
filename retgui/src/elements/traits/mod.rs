mod as_element;
mod deep_clone;
mod element;
mod element_data;
mod element_internals;

pub use as_element::AsElement;
pub(crate) use deep_clone::clone_element;
pub use element::Element;
pub use element_data::ElementData;
/// Note: this could be hidden behind a custom elements feature.
pub use element_internals::ElementInternals;
pub(crate) use element_internals::set_focus_outline_visible;
