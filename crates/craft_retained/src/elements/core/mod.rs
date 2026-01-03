mod element_data;
mod element_internals;

pub use element_data::ElementData;
/// Note: this could be hidden behind a custom elements feature.
pub use element_internals::ElementInternals;
pub(crate) use element_internals::resolve_clip_for_scrollable;
