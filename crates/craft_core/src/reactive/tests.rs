use std::any::Any;
use crate::components::{Component, ComponentId, ComponentSpecification, Context};
use crate::elements::element::ElementBoxed;
use crate::elements::{Container, Text};
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::reactive::element_id::reset_unique_element_id;
use crate::reactive::element_state_store::ElementStateStore;
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::diff_trees;
use crate::text::text_context::TextContext;
use crate::window_context::WindowContext;
use crate::{GlobalState, ReactiveTree};
use std::collections::{HashSet, VecDeque};
use crate::reactive::tracked_changes::TrackedChanges;

#[test]
fn diff_trees_same_tag_same_id_are_equal() {
    reset_unique_element_id();

    let mut text_context = TextContext::new();

    let initial_view = Container::new().component().push(Text::new("Foo").component());
    let updated_view = Container::new().component().push(Text::new("Foo").component());

    let root_element: ElementBoxed = Container::new().into();

    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();

    let mut window_context = WindowContext::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let initial_tree = diff_trees(
        initial_view,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let updated_tree = diff_trees(
        updated_view,
        root_element.clone(),
        Some(initial_tree.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let initial_id = &initial_tree.component_tree.children[0].children[0].id;
    let updated_id = &updated_tree.component_tree.children[0].children[0].id;

    assert_eq!(initial_id, updated_id, "Elements with identical content tags and positions have the same id.");
}

#[test]
fn diff_trees_after_one_iteration_adjacent_nodes_different_ids() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component());
    let root_node_2 =
        Container::new().component().push(Text::new("Foo").component()).push(Text::new("Bar").component());

    let root_element: ElementBoxed = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();

    let mut window_context = WindowContext::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(tree_1.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_ne!(initial_id, updated_id, "Elements in different positions should have different ids.");
}

#[test]
fn remove_unused_element_state_after_removal_is_state_deleted() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_component_1 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_component_2 = Container::new().component();
    let root_element: ElementBoxed = Container::new().into();

    let mut reactive_tree = ReactiveTree::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_component_1,
        root_element.clone(),
        None,
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let text_element_id = tree_1.component_tree.children[0].children[0].id;

    reactive_tree.component_tree = Some(tree_1.component_tree);
    reactive_tree.element_tree = Some(tree_1.element_tree.internal);
    reactive_tree.element_ids = tree_1.element_ids;
    reactive_tree.component_ids = tree_1.component_ids;

    let old_element_ids: HashSet<ComponentId> = reactive_tree.element_ids.clone();

    let tree_2 = diff_trees(
        root_component_2,
        root_element.clone(),
        Some(reactive_tree.component_tree.as_ref().unwrap().clone()),
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    reactive_tree.component_tree = Some(tree_2.component_tree);
    reactive_tree.element_tree = Some(tree_2.element_tree.internal);
    reactive_tree.element_ids = tree_2.element_ids;
    reactive_tree.component_ids = tree_2.component_ids;

    reactive_tree.element_state.remove_unused_state(&old_element_ids, &reactive_tree.element_ids);

    assert!(
        !reactive_tree.element_state.storage.contains_key(&text_element_id),
        "Unmounted elements should have their state removed."
    );
}

#[derive(Default)]
struct DummyComponent {}

impl Component for DummyComponent {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {
        Text::new("dummy").component()
    }
}

#[test]
fn remove_unused_component_state_after_removal_is_state_deleted() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_component_1 =
        Container::new().component().push(Text::new("Foo").component().key("key_1")).push(DummyComponent::component());
    let root_component_2 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_element: ElementBoxed = Container::new().into();

    let mut reactive_tree = ReactiveTree::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_component_1,
        root_element.clone(),
        None,
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let dummy_component_id = tree_1.component_tree.children[0].children[1].id;

    reactive_tree.component_tree = Some(tree_1.component_tree);
    reactive_tree.element_tree = Some(tree_1.element_tree.internal);
    reactive_tree.element_ids = tree_1.element_ids;
    reactive_tree.component_ids = tree_1.component_ids;

    let old_component_ids: HashSet<ComponentId> = reactive_tree.component_ids.clone();

    let tree_2 = diff_trees(
        root_component_2,
        root_element.clone(),
        Some(reactive_tree.component_tree.as_ref().unwrap().clone()),
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    reactive_tree.component_tree = Some(tree_2.component_tree);
    reactive_tree.element_tree = Some(tree_2.element_tree.internal);
    reactive_tree.element_ids = tree_2.element_ids;
    reactive_tree.component_ids = tree_2.component_ids;

    reactive_tree.user_state.remove_unused_state(&old_component_ids, &reactive_tree.component_ids);

    assert!(
        !reactive_tree.user_state.storage.contains_key(&dummy_component_id),
        "Unmounted components should have their state removed."
    );
}

#[test]
fn diff_trees_after_one_iteration_same_key_different_position_same_id() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_node_2 =
        Container::new().component().push(Text::new("Bar").component()).push(Text::new("Foo").component().key("key_1"));

    let root_element: ElementBoxed = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(tree_1.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_eq!(initial_id, updated_id, "Elements in different positions with the same key, should have the same id.");
}

#[test]
fn diff_trees_after_one_iteration_same_position_different_component_keys_different_id() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_node_1 = DummyComponent::component().key("key_1");
    let root_node_2 = DummyComponent::component().key("key_2");

    let root_element: ElementBoxed = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(tree_1.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let initial_id = &tree_1.component_tree.children[0].id;
    let updated_id = &tree_2.component_tree.children[0].id;

    assert_ne!(
        initial_id, updated_id,
        "Components in the same position with different keys, should have different ids."
    );
}

#[test]
fn diff_trees_after_one_iteration_same_position_different_components_different_child_element_id() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_node_1 = DummyComponent::component().key("key_1");
    let root_node_2 = DummyComponent::component().key("key_2");

    let root_element: ElementBoxed = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();
    
    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(tree_1.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes,
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[0].id;

    assert_ne!(
        initial_id, updated_id,
        "Different Components in the same position should not have the same element child id."
    );
}

#[test]
fn diff_trees_elements_swapped_keys_should_swap_ids() {
    let mut text_context = TextContext::new();
    reset_unique_element_id();

    let root_node_1 = Container::new()
        .component()
        .push(Text::new("A").component().key("a"))
        .push(Text::new("B").component().key("b"));

    let root_node_2 = Container::new()
        .component()
        .push(Text::new("B").component().key("b"))
        .push(Text::new("A").component().key("a"))
        ;

    let root_element: ElementBoxed = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
    let mut window_context = WindowContext::new();
    let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();
    let mut tracked_changes = TrackedChanges::default();

    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element,
        Some(tree_1.component_tree.clone()),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        false,
        &mut text_context,
        1.0,
        &mut window_context,
        &mut update_queue,
        &mut tracked_changes
    );

    let a_id_1 = &tree_1.component_tree.children[0].children[0].id;
    let b_id_1 = &tree_1.component_tree.children[0].children[1].id;

    let a_id_2 = &tree_2.component_tree.children[0].children[1].id;
    let b_id_2 = &tree_2.component_tree.children[0].children[0].id;
    
    assert_eq!(a_id_1, a_id_2, "Element with key 'a' should retain its ID after being moved");
    assert_eq!(b_id_1, b_id_2, "Element with key 'b' should retain its ID after being moved");
}
