use crate::elements::DynElement;

/// Gives an arena-owned node access to its shared element data.
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
