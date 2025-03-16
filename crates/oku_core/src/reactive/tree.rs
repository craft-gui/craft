use crate::components::component::{ComponentId, ComponentOrElement, ComponentSpecification, UpdateFn};
use crate::components::{Props, UpdateResult};
use crate::elements::container::ContainerState;
use crate::elements::element::{Element, ElementBox};
use crate::events::{Event, Message, OkuMessage};
use crate::reactive::element_id::create_unique_element_id;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::reactive::state_store::{StateStore, StateStoreItem};

use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::GlobalState;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct ComponentTreeNode {
    pub is_element: bool,
    pub key: Option<String>,
    pub tag: String,
    pub update: UpdateFn,
    pub children: Vec<ComponentTreeNode>,
    pub children_keys: HashMap<String, ComponentId>,
    pub id: ComponentId,
    pub(crate) parent_id: Option<ComponentId>,
    pub props: Props,
}

#[derive(Clone)]
struct TreeVisitorNode {
    component_specification: ComponentSpecification,
    parent_element_ptr: *mut dyn Element,
    parent_component_node: *mut ComponentTreeNode,
    old_component_node: Option<*const ComponentTreeNode>,
}

impl ComponentTreeNode {
    #[allow(dead_code)]
    pub fn print_tree(&self) {
        let mut elements: Vec<(&ComponentTreeNode, usize, bool)> = vec![(self, 0, true)];
        while let Some((element, indent, is_last)) = elements.pop() {
            let mut prefix = String::new();
            for _ in 0..indent {
                prefix.push_str("  ");
            }
            if is_last {
                prefix.push_str("└─");
            } else {
                prefix.push_str("├─");
            }
            println!(
                "{} , Tag: {}, Id: {}, Key: {:?}, Parent: {:?}",
                prefix, element.tag, element.id, element.key, element.parent_id
            );
            let children = &element.children;
            for (i, child) in children.iter().enumerate().rev() {
                let is_last = i == children.len() - 1;
                elements.push((child, indent + 1, is_last));
            }
        }
    }
}
fn dummy_update(
    _state: &mut StateStoreItem,
    _global_state: &mut GlobalState,
    _props: Props,
    _message: Event,
) -> UpdateResult {
    UpdateResult::new()
}

pub struct DiffTreesResult {
    pub(crate) component_tree: ComponentTreeNode,
    pub(crate) element_tree: ElementBox,
    pub(crate) component_ids: HashSet<ComponentId>,
    pub(crate) element_ids: HashSet<ComponentId>,
    pub(crate) pointer_captures: HashMap<i64, ComponentId>,
}

/// Creates a new Component tree and Element tree from a ComponentSpecification.
/// The ids of the Component tree are stable across renders.
pub(crate) fn diff_trees(
    component_specification: ComponentSpecification,
    mut root_element: ElementBox,
    old_component_tree: Option<&ComponentTreeNode>,
    user_state: &mut StateStore,
    global_state: &mut GlobalState,
    element_state: &mut ElementStateStore,
    reload_fonts: bool,
) -> DiffTreesResult {
    unsafe {
        let mut component_tree = ComponentTreeNode {
            is_element: false,
            key: None,
            tag: "root".to_string(),
            update: dummy_update,
            children: vec![],
            children_keys: HashMap::new(),
            id: 0,
            parent_id: None,
            props: Props::new(()),
        };

        // Make sure to set a default state for the root.
        element_state.storage.insert(
            0,
            ElementStateStoreItem {
                base: Default::default(),
                data: Box::new(ContainerState::default()),
            },
        );

        let mut old_component_tree_as_ptr = old_component_tree.map(|old_root| old_root as *const ComponentTreeNode);

        // HACK: This is a workaround to get the first child of the old component tree because we start at the first level on the new tree.
        // This is because the root of the component tree is not a component, but a dummy node.
        if old_component_tree_as_ptr.is_some() {
            old_component_tree_as_ptr =
                Some((*old_component_tree_as_ptr.unwrap()).children.first().unwrap() as *const ComponentTreeNode);
        }

        let component_root: *mut ComponentTreeNode = &mut component_tree as *mut ComponentTreeNode;

        let root_spec = ComponentSpecification {
            component: ComponentOrElement::Element(root_element.clone()),
            key: None,
            props: None,
            children: vec![component_specification],
        };

        let mut new_component_ids: HashSet<ComponentId> = HashSet::new();
        let mut new_element_ids: HashSet<ComponentId> = HashSet::new();
        let mut pointer_captures: HashMap<i64, ComponentId> = HashMap::new();

        let mut to_visit: Vec<TreeVisitorNode> = vec![TreeVisitorNode {
            component_specification: root_spec.children[0].clone(),
            parent_element_ptr: root_element.internal.as_mut() as *mut dyn Element,
            parent_component_node: component_root,
            old_component_node: old_component_tree_as_ptr,
        }];

        while let Some(tree_node) = to_visit.pop() {
            let old_tag = tree_node.old_component_node.map(|old_node| (*old_node).tag.as_str());
            let mut parent_element_ptr = tree_node.parent_element_ptr;
            let parent_component_ptr = tree_node.parent_component_node;

            let new_spec = tree_node.component_specification;

            match new_spec.component {
                ComponentOrElement::Element(element) => {
                    // Create the new element node.
                    let mut element = element;

                    // Store the new tag, i.e. the element's name.
                    let new_tag = element.internal.name().to_string();

                    let mut should_update = false;
                    let id = match old_tag {
                        Some(old_tag) if new_tag == old_tag => {
                            should_update = true;
                            (*tree_node.old_component_node.unwrap()).id
                        }
                        _ => create_unique_element_id(),
                    };
                    element.internal.set_component_id(id);
                    // Collect the element id for later use.
                    new_element_ids.insert(id);

                    if should_update {
                        // Collect the pointer captures.
                        let base_state = element.internal.get_base_state(element_state);
                        // FIXME: Collect pointer captures with the correct device id.
                        for is_captured in base_state.base.pointer_capture.values() {
                            if *is_captured {
                                pointer_captures.insert(DUMMY_DEVICE_ID /*device_id*/, id);
                            }
                        }

                        element.internal.update_state(element_state, reload_fonts);
                    } else {
                        let state = element.internal.initialize_state();
                        element_state.storage.insert(id, state);
                    }

                    // Move the new element into it's parent and set the parent element to be the new element.
                    tree_node.parent_element_ptr.as_mut().unwrap().children_mut().push(element);
                    parent_element_ptr = tree_node
                        .parent_element_ptr
                        .as_mut()
                        .unwrap()
                        .children_mut()
                        .last_mut()
                        .unwrap()
                        .internal
                        .as_mut();

                    let new_component_node = ComponentTreeNode {
                        is_element: true,
                        key: new_spec.key,
                        tag: new_tag,
                        update: dummy_update,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                        parent_id: Some((*parent_component_ptr).id),
                        props: Props::new(()),
                    };

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode =
                        (*tree_node.parent_component_node).children.last_mut().unwrap();

                    // Get the old children of the old component node.
                    let mut olds: Vec<*const ComponentTreeNode> = vec![];
                    if tree_node.old_component_node.is_some() {
                        for child in (*tree_node.old_component_node.unwrap()).children.iter() {
                            olds.push(child as *const ComponentTreeNode);
                        }
                    }

                    let mut new_to_visits: Vec<TreeVisitorNode> = vec![];
                    // Add the children of the new element to the to visit list.
                    for (index, child) in new_spec.children.into_iter().enumerate() {
                        // Find old child by key and if no key is found, find by index.
                        let key = &child.key;

                        let mut index = index;

                        for (old_index, old_child) in olds.iter().enumerate() {
                            let old_key = (*(*old_child)).key.as_deref();

                            if old_key == key.as_deref() {
                                if old_key.is_none() || key.is_none() {
                                    continue;
                                }
                                index = old_index;
                                break;
                            }
                        }

                        new_to_visits.push(TreeVisitorNode {
                            component_specification: child,
                            parent_element_ptr,
                            parent_component_node: new_component_pointer,
                            old_component_node: olds.get(index).copied(),
                        });
                    }

                    to_visit.extend(new_to_visits.into_iter().rev());
                }
                ComponentOrElement::ComponentSpec(component_data) => {
                    let children_keys = (*parent_component_ptr).children_keys.clone();
                    let props = new_spec.props.unwrap_or((component_data.default_props)());

                    let mut should_update = false;
                    let id: ComponentId =
                        if new_spec.key.is_some() && children_keys.contains_key(new_spec.key.as_deref().unwrap()) {
                            *(children_keys.get(new_spec.key.as_deref().unwrap()).unwrap())
                        } else if let Some(old_tag) = old_tag {
                            if component_data.tag.as_str() == old_tag {
                                // If the old tag is the same as the new tag, we can reuse the old id.
                                should_update = true;
                                (*tree_node.old_component_node.unwrap()).id
                            } else {
                                create_unique_element_id()
                            }
                        } else {
                            create_unique_element_id()
                        };

                    // Collect the component id for later use.
                    new_component_ids.insert(id);

                    if !should_update {
                        let default_state = (component_data.default_state)();
                        user_state.storage.insert(id, default_state);
                        let state_mut = user_state.storage.get_mut(&id).unwrap().as_mut();

                        (component_data.update_fn)(
                            state_mut,
                            global_state,
                            props.clone(),
                            Event::new(Message::OkuMessage(OkuMessage::Initialized)),
                        );
                    }

                    let state = user_state.storage.get(&id);
                    let state = state.unwrap().as_ref();
                    let new_component = (component_data.view_fn)(state, global_state, props.clone(), new_spec.children);

                    // Add the current child id to the children_keys hashmap in the parent.
                    if let Some(key) = new_spec.key.clone() {
                        parent_component_ptr.as_mut().unwrap().children_keys.insert(key, id);
                    }

                    let new_component_node = ComponentTreeNode {
                        is_element: false,
                        key: new_spec.key,
                        tag: component_data.tag,
                        update: component_data.update_fn,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                        parent_id: Some((*parent_component_ptr).id),
                        props,
                    };

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode =
                        (*tree_node.parent_component_node).children.last_mut().unwrap();

                    // Get the old component node or none.
                    // NOTE: ComponentSpecs can only have one child.
                    let old_component_tree = tree_node.old_component_node.and_then(|old_node| {
                        (*old_node).children.first().map(|child| child as *const ComponentTreeNode)
                    });

                    // Add the computed component spec to the to visit list.
                    to_visit.push(TreeVisitorNode {
                        component_specification: new_component,
                        parent_element_ptr,
                        parent_component_node: new_component_pointer,
                        old_component_node: old_component_tree,
                    });
                }
            };
        }

        DiffTreesResult {
            component_tree,
            element_tree: root_element,
            element_ids: new_element_ids,
            component_ids: new_component_ids,
            pointer_captures,
        }
    }
}
