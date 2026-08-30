use crate::elements::DynElement;

/// Used as a super trait and forces implementations to
/// support the retrieval and mutation of `ElementData`(struct).
pub trait ElementData {
    /// Get a shared reference to this element's common element data.
    fn element_data(&self) -> &crate::elements::element_data::ElementData;

    /// Get a mutable reference to this element's common element data.
    fn element_data_mut(&mut self) -> &mut crate::elements::element_data::ElementData;

    /// Returns a unique id for the element.
    fn id(&self) -> u64 {
        self.element_data().internal_id
    }

    /// Returns the element's parent element.
    fn parent(&self) -> Option<DynElement> {
        self.element_data()
            .parent
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .map(DynElement::new)
    }

    /// Returns the element's children.
    fn get_children(&self) -> &[DynElement] {
        self.element_data().children.as_slice()
    }
}
