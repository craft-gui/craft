use std::collections::VecDeque;
use std::sync::Arc;
use winit::event::{ElementState, Ime, MouseButton};
use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::geometry::Point;
use crate::{GlobalState, ReactiveTree, WindowContext};
use crate::components::{ComponentId, Event, PointerCapture};
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::Element;
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::reactive::fiber_node::FiberNode;
use crate::reactive::tree::ComponentTreeNode;
use crate::resource_manager::ResourceManager;
use crate::text::text_context::TextContext;

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_event(
    message: &Message,
    dispatch_type: EventDispatchType,
    _resource_manager: &mut Arc<ResourceManager>,
    mouse_position: Option<Point>,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    text_context: &mut Option<TextContext>,
    window_context: &mut WindowContext,
    is_style: bool
) {
    let mut effects: Vec<(EventDispatchType, Message)> = Vec::new();

    let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: FiberNode = FiberNode {
        element: Some(current_element_tree.as_ref()),
        component: Some(reactive_tree.component_tree.as_ref().unwrap()),
    };

    let is_pointer_event = matches!(
        message,
        Message::CraftMessage(CraftMessage::PointerMovedEvent(_))
            | Message::CraftMessage(CraftMessage::PointerButtonEvent(_))
    );
    let is_ime_event = matches!(
        message,
        Message::CraftMessage(CraftMessage::ImeEvent(Ime::Enabled))
            | Message::CraftMessage(CraftMessage::ImeEvent(Ime::Disabled))
    );

    #[derive(Clone)]
    struct Target<'a> {
        component_id: ComponentId,
        layout_order: usize,
        overlay_depth: usize,
        element: Option<&'a dyn Element>,
        component: Option<&'a ComponentTreeNode>
    }

    match dispatch_type {
        EventDispatchType::Bubbling => {
            let mut targets: VecDeque<Target> = VecDeque::new();
            
            /////////////////////////////////////////
            // A,0                                 //
            //   /////////////////////////         //
            //   // B,1                 //         //
            //   //   ///////////       //         //
            //   //   //       //       //         //
            //   //   //  C,2  //       //         //
            //   //   //       //       //         //
            //   //   ///////////       //         //
            //   //                     //         //
            //   /////////////////////////         //
            //                                     //
            /////////////////////////////////////////

            // Collect all possible target elements in reverse order.
            // Nodes added last are usually on top, so these elements are in visual order.


            for (fiber_node, overlay_depth) in fiber.dfs_with_overlay_depth() {
                
                if let Some(element) = fiber_node.element {
                    let in_bounds = mouse_position.is_some() && element.in_bounds(mouse_position.unwrap());
                    let mut should_pass_hit_test = in_bounds;

                    // Bypass the hit test result if pointer capture is turned on for the current element.
                    if is_pointer_event || is_ime_event {
                        if let Some(element_id) = reactive_tree.pointer_captures.get(&DUMMY_DEVICE_ID) {
                            if *element_id == element.component_id() {
                                should_pass_hit_test = true;
                            }
                        }
                    }

                    if should_pass_hit_test {
                        let mut parent_component_targets: VecDeque<Target> = VecDeque::new();
                        let current_target_component = reactive_tree
                            .component_tree
                            .as_ref()
                            .unwrap()
                            .pre_order_iter()
                            .find(|node| node.id == element.component_id())
                            .unwrap();
                        let mut to_visit = Some(current_target_component);
                        while let Some(node) = to_visit {
                            if node.id == 0 {
                                break;
                            }
                            if !node.is_element {

                                parent_component_targets.push_back(Target {
                                    component_id: node.id,
                                    layout_order: element.element_data().layout_item.layout_order as usize,
                                    overlay_depth,
                                    element: Some(element),
                                    component: Some(node),
                                });
                                
                                if let Some(parent_id) = node.parent_id {
                                    to_visit = reactive_tree
                                        .component_tree
                                        .as_ref()
                                        .unwrap()
                                        .pre_order_iter()
                                        .find(|node2| node2.id == parent_id);
                                } else{
                                    to_visit = None;
                                }
                            } else if node.parent_id.is_none() {
                                to_visit = None;
                            } else if node.id == element.component_id() {
                                let parent_id = node.parent_id.unwrap();
                                to_visit = reactive_tree
                                    .component_tree
                                    .as_ref()
                                    .unwrap()
                                    .pre_order_iter()
                                    .find(|node2| node2.id == parent_id);
                            } else {
                                to_visit = None;
                            }
                        }
                        
                        for parent in parent_component_targets.drain(..).rev() {
                            targets.push_back(parent);
                        }

                        targets.push_back(Target {
                            component_id: element.component_id(),
                            layout_order: element.element_data().layout_item.layout_order as usize,
                            overlay_depth,
                            element: Some(element),
                            component: None,
                        });
                    }
                }
            }

            // The targets should be [(2, Some(c)), (1, Some(b)), (0, Some(a))].
            if targets.is_empty() {
                return;
            }

            // The target is always the first node (2, Some(c)).
            let mut tmp_targets: Vec<Target> = targets.clone().into_iter().collect();
            tmp_targets.sort_by(|a, b| b.layout_order.cmp(&a.layout_order)); // Sort using the layout order. (u32)
            tmp_targets.sort_by(|a, b| b.overlay_depth.cmp(&a.overlay_depth)); // Sort using the overlay depth order. (u32)
            targets = VecDeque::from(tmp_targets);

            let mut element_events: VecDeque<(CraftMessage, &dyn Element)> = VecDeque::new();

            let target = targets[0].clone();
            let mut propagate = true;
            let mut prevent_defaults = false;
            for current_target in targets.iter() {
                if !propagate {
                    break;
                }

                // Get the element's component tree node.
                let current_target_component = reactive_tree
                    .component_tree
                    .as_ref()
                    .unwrap()
                    .pre_order_iter()
                    .find(|node| node.id == current_target.component_id)
                    .unwrap();
                
                if current_target.component.is_some() {
                    continue;
                }

                // Search for the closest non-element ancestor.
                let mut closest_ancestor_component: Option<&ComponentTreeNode> = None;

                let mut to_visit = Some(current_target_component);
                while let Some(node) = to_visit {
                    if !node.is_element {
                        closest_ancestor_component = Some(node);
                        to_visit = None;
                    } else if node.parent_id.is_none() {
                        to_visit = None;
                    } else {
                        let parent_id = node.parent_id.unwrap();
                        to_visit = reactive_tree
                            .component_tree
                            .as_ref()
                            .unwrap()
                            .pre_order_iter()
                            .find(|node2| node2.id == parent_id);
                    }
                }

                // Dispatch the event to the element's component.
                if let Some(node) = closest_ancestor_component {
                    let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
                    let mut event = Event::with_window_context(window_context.clone());
                    event.target = Some(target.element.unwrap());
                    event.current_target = Some(current_target.element.unwrap());
                    (node.update)(
                        state,
                        global_state,
                        node.props.clone(),
                        &mut event,
                        message,
                    );

                    if !event.prevent_defaults && event.propagate {
                        if let Some(ref result_message) = event.result_message {
                            element_events.push_back((result_message.clone(), *event.target.as_ref().unwrap()));
                        }
                    }

                    *window_context = event.window.clone();
                    effects.append(&mut event.effects);
                    propagate = propagate && event.propagate;
                    let element_state =
                        &mut reactive_tree.element_state.storage.get_mut(&current_target.component_id).unwrap().base;
                    match event.pointer_capture {
                        PointerCapture::None => {}
                        PointerCapture::Set => {
                            element_state.pointer_capture.insert(DUMMY_DEVICE_ID, true);
                        }
                        PointerCapture::Unset => {
                            element_state.pointer_capture.remove(&DUMMY_DEVICE_ID);
                        }
                    }
                    prevent_defaults = prevent_defaults || event.prevent_defaults;
                    if event.future.is_some() {
                        reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                            node.id,
                            node.update,
                            event,
                            node.props.clone(),
                        ));
                    }
                }
            }

            for element_state in reactive_tree.element_state.storage.values_mut() {
                if let Message::CraftMessage(message) = &message {
                    match message {
                        CraftMessage::PointerMovedEvent(..) => {
                            element_state.base.hovered = false;
                        }
                        CraftMessage::PointerButtonEvent(pointer_button) => {
                            if pointer_button.button.mouse_button() == MouseButton::Left && pointer_button.state == ElementState::Released {
                                element_state.base.active = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Handle element events if prevent defaults was not set to true.
            //let mut first_element = true;
            if !prevent_defaults {
                for target in targets.iter() {
                    let mut propagate = true;
                    let mut prevent_defaults = false;

                    for element in current_element_tree.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                        if !propagate {
                            break;
                        }
                        if element.component_id() == target.component_id {
                            if let Message::CraftMessage(event) = message {
                                let mut res = Event::new();
                                element.on_event(
                                    event,
                                    &mut reactive_tree.element_state,
                                    text_context.as_mut().unwrap(),
                                    // first_element && is_style. For only the first element.
                                    is_style,
                                    &mut res,
                                );
                                //first_element = false;

                                if let Some(result_message) = res.result_message {
                                    element_events.push_back((result_message, *element));
                                }

                                propagate = propagate && res.propagate;
                                prevent_defaults = prevent_defaults || res.prevent_defaults;
                            }
                        }
                    }
                }
            }

            for (message, target_element) in element_events.iter() {
                let mut propagate = true;
                let mut prevent_defaults = false;
                for node in targets.iter() {
                    let current_target = reactive_tree
                        .component_tree
                        .as_ref()
                        .unwrap()
                        .pre_order_iter()
                        .find(|node2| node2.id == node.component_id)
                        .unwrap();


                    if !propagate {
                        break;
                    }

                    let mut event = Event::with_window_context(window_context.clone());
                    if current_target.is_element {
                        if let Some(element) = current_element_tree
                            .pre_order_iter()
                            .collect::<Vec<&dyn Element>>()
                            .iter()
                            .find(|node2| node2.element_data().component_id == current_target.id) {
                            element.on_event(
                                message,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                // first_element && is_style. For only the first element.
                                is_style,
                                &mut event,
                            );
                        }
                    } else {
                        let state = reactive_tree.user_state.storage.get_mut(&current_target.id).unwrap().as_mut();
                        // For element events the target and current target
                        // are the element the event was dispatched from.
                        event.target = Some(*target_element);
                        event.current_target = Some(*target_element);
                        (current_target.update)(
                            state,
                            global_state,
                            current_target.props.clone(),
                            &mut event,
                            &Message::CraftMessage(message.clone()),
                        );
                    }
                    *window_context = event.window.clone();
                    effects.append(&mut event.effects);
                    propagate = propagate && event.propagate;
                    prevent_defaults = prevent_defaults || event.prevent_defaults;
                    if event.future.is_some() {
                        reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                            current_target.id,
                            current_target.update,
                            event,
                            current_target.props.clone(),
                        ));
                    }
                }
            }
        }
        EventDispatchType::Direct(id) => {
            for node in fiber.pre_order_iter().collect::<Vec<FiberNode>>().iter() {
                if let Some(element) = node.element {
                    if element.component_id() == id {
                        if let Message::CraftMessage(message) = message {
                            let mut res = Event::new();
                            element.on_event(
                                message,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                false,
                                &mut res,
                            );

                            effects.append(&mut res.effects);
                        }

                        return;
                    }
                }
                if let Some(component) = node.component {
                    if component.id == id {
                        let state = reactive_tree.user_state.storage.get_mut(&component.id).unwrap().as_mut();
                        let mut event = Event::with_window_context(window_context.clone());
                        event.current_target = None;
                        event.target = None;
                        (component.update)(
                            state,
                            global_state,
                            component.props.clone(),
                            &mut event,
                            message,
                        );
                        *window_context = event.window.clone();
                        effects.append(&mut event.effects);
                        if event.future.is_some() {
                            reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                                component.id,
                                component.update,
                                event,
                                component.props.clone(),
                            ));
                        }

                        return;
                    }
                }
            }
        }
    }

    // Handle effects.
    for (dispatch_type, message) in effects.iter() {
        dispatch_event(
            message,
            *dispatch_type,
            _resource_manager,
            mouse_position,
            reactive_tree,
            global_state,
            text_context,
            window_context,
            false
        );
    }
}