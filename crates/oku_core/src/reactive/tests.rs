use crate::elements::{Container, Text};
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::create_trees_from_render_specification;

#[test]
fn create_trees_from_render_specification_test() {
    let x = Container::new().component().push(Text::new("Foo").component());

    let root = Container::new().into();

    let mut user_state = StateStore::default();
    let mut element_state = StateStore::default();

    let res = create_trees_from_render_specification(x, root, None, &mut user_state, &mut element_state);
}
