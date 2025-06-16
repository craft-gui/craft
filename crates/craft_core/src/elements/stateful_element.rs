use crate::elements::base_element_state::BaseElementState;
use crate::elements::Element;
use crate::elements::text_input::TextInputState;
use crate::reactive::element_state_store::ElementStateStore;

pub trait StatefulElement<State: 'static> : Element {

    fn state<'a>(&self, element_state: &'a ElementStateStore) -> &'a State {
        element_state.storage.get(&self.element_data().component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut State {
        element_state.storage.get_mut(&self.element_data().component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }

    fn state_and_base_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> (&'a mut State, &'a mut BaseElementState) {
        let state = element_state.storage.get_mut(&self.element_data().component_id).unwrap();
        (state.data.as_mut().downcast_mut().unwrap(), &mut state.base)
    }

}