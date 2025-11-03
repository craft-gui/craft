mod element_internals;
mod element_data;

/// Note: this could be hidden behind a custom elements feature.
pub use element_internals::ElementInternals;
pub(crate) use element_internals::resolve_clip_for_scrollable;
pub use element_data::ElementData;