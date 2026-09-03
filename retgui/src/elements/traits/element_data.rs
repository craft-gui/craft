use crate::elements::DynElement;

/// Used as a super trait and forces implementations to
/// support the retrieval and mutation of `ElementData`(struct).
pub trait ElementNodeData {
    fn element_data(&self) -> &crate::elements::element_data::ElementData;

    fn element_data_mut(&mut self) -> &mut crate::elements::element_data::ElementData;

    fn parent(&self) -> Option<DynElement> {
        self.element_data().parent
    }

    fn get_children(&self) -> &[DynElement] {
        self.element_data().children.as_slice()
    }
}
