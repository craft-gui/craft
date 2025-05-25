use crate::components::{Event, PointerCapture};
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::Element;
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::geometry::Point;
use crate::reactive::fiber_tree;
use crate::reactive::fiber_tree::FiberNode;
use crate::reactive::tree::ComponentTreeNode;
use crate::resource_manager::ResourceManager;
use crate::text::text_context::TextContext;
use crate::{GlobalState, ReactiveTree, WindowContext};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use winit::event::{Ime};
use craft_logging::{span, Level};

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
    is_style: bool,
) {
    let span = span!(Level::INFO, "dispatch event");
    let _enter = span.enter();

    let mut effects: Vec<(EventDispatchType, Message)> = Vec::new();

    let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: Rc<RefCell<FiberNode>> =
        fiber_tree::new(reactive_tree.component_tree.as_ref().unwrap(), current_element_tree.as_ref());

    let mut nodes: Vec<Rc<RefCell<FiberNode>>> = Vec::new();
    let mut to_visit: Vec<Rc<RefCell<FiberNode>>> = vec![fiber.clone()];

    while let Some(node) = to_visit.pop() {
        nodes.push(node.clone());
        for child in node.borrow().children.iter().rev() {
            to_visit.push(child.clone());
        }
    }

    let is_pointer_event = matches!(
        message,
        Message::CraftMessage(CraftMessage::PointerMovedEvent(_))
            | Message::CraftMessage(CraftMessage::PointerButtonUp(_))
            | Message::CraftMessage(CraftMessage::PointerButtonDown(_))
    );
    let is_ime_event = matches!(
        message,
        Message::CraftMessage(CraftMessage::ImeEvent(Ime::Enabled))
            | Message::CraftMessage(CraftMessage::ImeEvent(Ime::Disabled))
    );

    match dispatch_type {
        EventDispatchType::Bubbling => {
            nodes.retain_mut(|node| node.borrow().element.is_some());

            // Sort by layout order descending.
            nodes.sort_by(|a, b| {
                b.borrow()
                    .element
                    .unwrap()
                    .element_data()
                    .layout_item
                    .layout_order
                    .cmp(&a.borrow().element.unwrap().element_data().layout_item.layout_order)
            });

            // Sort by overlay order descending.
            nodes.sort_by(|a, b| b.borrow().overlay_order.cmp(&a.borrow().overlay_order));

            // 1. Do a hit test to find the target element.
            // We order by the overlay depth descending and layout order descending.
            let mut target: Option<Rc<RefCell<FiberNode>>> = None;
            let mut targets: VecDeque<Rc<RefCell<FiberNode>>> = VecDeque::new();

            for node in nodes {
                if let Some(element) = node.borrow().element {
                    let should_pass_hit_test = mouse_position.is_some() && element.in_bounds(mouse_position.unwrap());

                    // The first element to pass the hit test should be the target.
                    if should_pass_hit_test && target.is_none() {
                        target = Some(node.clone());
                    }

                    // Unless another element has pointer capture.
                    if is_pointer_event || is_ime_event {
                        if let Some(element_id) = reactive_tree.pointer_captures.get(&DUMMY_DEVICE_ID) {
                            if *element_id == element.component_id() {
                                target = Some(node.clone());
                                break;
                            }
                        }
                    }
                }
            }
            if target.is_none() {
                return;
            }
            let target = target.unwrap();

            let mut current_target = Some(target.clone());
            while current_target.is_some() {
                targets.push_back(current_target.clone().unwrap());
                current_target = current_target.clone().unwrap().borrow().parent.clone();
            }
            
            if targets.is_empty() {
                return;
            }
            
            let mut element_events: VecDeque<(CraftMessage, &dyn Element)> = VecDeque::new();

            let target = targets[0].clone();
            let mut propagate = true;
            let mut prevent_defaults = false;
            for current_target in targets.iter() {
                if !propagate {
                    break;
                }
                
                if current_target.borrow().element.is_none() {
                    continue;
                }

                // Search for the closest non-element ancestor.
                let mut closest_ancestor_component: Option<&ComponentTreeNode> = None;

                let mut current_component = Some(current_target.clone());
                while current_component.is_some() {
                    if current_component.clone().unwrap().borrow().element.is_none() {
                        closest_ancestor_component = Some(current_component.unwrap().borrow().component);
                        break;
                    }
                    current_component = current_component.clone().unwrap().borrow().parent.clone();
                }

                // Dispatch the event to the element's component.
                if let Some(node) = closest_ancestor_component {
                    let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
                    let mut event = Event::with_window_context(window_context.clone());
                    event.target = Some(target.borrow().element.unwrap());
                    event.current_target = Some(current_target.borrow().element.unwrap());
                    (node.update)(state, global_state, node.props.clone(), &mut event, message);

                    if !event.prevent_defaults && event.propagate {
                        if let Some(ref result_message) = event.result_message {
                            element_events.push_back((result_message.clone(), *event.target.as_ref().unwrap()));
                        }
                    }

                    *window_context = event.window.clone();
                    effects.append(&mut event.effects);
                    propagate = propagate && event.propagate;
                    let element_state = &mut reactive_tree
                        .element_state
                        .storage
                        .get_mut(&current_target.borrow().component.id)
                        .unwrap()
                        .base;
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
                        CraftMessage::PointerButtonUp(pointer_button) => {
                            if pointer_button.is_primary() {
                                element_state.base.active = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Handle element events if prevent defaults was not set to true.
            if !prevent_defaults {
                let mut propagate = true;
                for target in targets.iter() {
                    if !propagate || prevent_defaults {
                        break;
                    }
                    if let Some(element) = target.borrow().element {
                        if let Message::CraftMessage(event) = message {
                            let mut res = Event::new();
                            element.on_event(
                                event,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                is_style,
                                &mut res,
                            );

                            if let Some(result_message) = res.result_message {
                                element_events.push_back((result_message, element));
                            }

                            propagate = propagate && res.propagate;
                            prevent_defaults = prevent_defaults || res.prevent_defaults;
                        }
                    }
                }
            }

            for (message, target_element) in element_events.iter() {
                let mut propagate = true;
                let mut prevent_defaults = false;
                for node in targets.iter() {
                    let current_target = node.borrow();

                    if !propagate {
                        break;
                    }

                    let mut event = Event::with_window_context(window_context.clone());
                    if let Some(element) = current_target.element {
                        element.on_event(
                            message,
                            &mut reactive_tree.element_state,
                            text_context.as_mut().unwrap(),
                            // first_element && is_style. For only the first element.
                            is_style,
                            &mut event,
                        );
                    } else {
                        let state =
                            reactive_tree.user_state.storage.get_mut(&current_target.component.id).unwrap().as_mut();
                        // For element events the target and current target
                        // are the element the event was dispatched from.
                        event.target = Some(*target_element);
                        event.current_target = Some(*target_element);
                        (current_target.component.update)(
                            state,
                            global_state,
                            current_target.component.props.clone(),
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
                            current_target.component.id,
                            current_target.component.update,
                            event,
                            current_target.component.props.clone(),
                        ));
                    }
                }
            }
        }
        EventDispatchType::Direct(id) => {
            for node in nodes {
                if node.borrow().component.id == id {
                    if let Some(element) = node.borrow().element {
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

                        break;
                    } else {
                        let component = node.borrow().component;
                        let state = reactive_tree.user_state.storage.get_mut(&component.id).unwrap().as_mut();
                        let mut event = Event::with_window_context(window_context.clone());
                        event.current_target = None;
                        event.target = None;
                        (component.update)(state, global_state, component.props.clone(), &mut event, message);
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

                        break;
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
            false,
        );
    }
}
