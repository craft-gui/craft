mod element_data;
mod element_internals;
mod as_element;
mod element;
mod deep_clone;

pub use deep_clone::DeepClone;
pub use element_data::ElementData;
pub use as_element::AsElement;
pub use element::Element;
/// Note: this could be hidden behind a custom elements feature.
pub use element_internals::{ElementInternals, resolve_clip_for_scrollable};