use crate::elements::ElementInternals;

/// Used as a super trait in `Element`, so that the inner element can be retrieved.
pub trait AsElement {
    /// Provides access to the element.
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R;

    /// Provides mutable access to the element.
    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R;
}
