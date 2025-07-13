use crate::components::{Event, FocusAction, PointerCapture};
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::Element;
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::reactive::fiber_tree;
use crate::reactive::fiber_tree::FiberNode;
use crate::reactive::tree::ComponentTreeNode;
use crate::text::text_context::TextContext;
use crate::window_context::WindowContext;
use crate::{GlobalState, ReactiveTree};
use craft_logging::{span, Level};
use craft_primitives::geometry::Point;
use craft_resource_manager::ResourceManager;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use winit::event::Ime;

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
    let mut focus = FocusAction::None;
    let span = span!(Level::INFO, "dispatch event");
    let _enter = span.enter();

    let mut effects: Vec<(EventDispatchType, Message)> = Vec::new();

    {
        let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
            current_element_tree
        } else {
            return;
        };

        let fiber: Rc<RefCell<FiberNode>> =
            fiber_tree::new(reactive_tree.component_tree.as_ref().unwrap(), current_element_tree.as_ref());

        let mut nodes: Vec<Rc<RefCell<FiberNode>>> = Vec::new();
        let mut to_visit: Vec<Rc<RefCell<FiberNode>>> = vec![Rc::clone(&fiber)];

        while let Some(node_rc) = to_visit.pop() {
            let node_ref = node_rc.borrow();

            if node_ref.element.is_some() {
                nodes.push(Rc::clone(&node_rc));
            }

            for child in node_ref.children.iter().rev() {
                to_visit.push(Rc::clone(child));
            }
        }

        let is_pointer_event = matches!(
            message,
            Message::CraftMessage(CraftMessage::PointerMovedEvent(_))
                | Message::CraftMessage(CraftMessage::PointerButtonUp(_))
                | Message::CraftMessage(CraftMessage::PointerButtonDown(_))
        );
        let is_keyboard_event = matches!(message, Message::CraftMessage(CraftMessage::KeyboardInputEvent(_)));
        let is_ime_event = matches!(
            message,
            Message::CraftMessage(CraftMessage::ImeEvent(Ime::Enabled))
                | Message::CraftMessage(CraftMessage::ImeEvent(Ime::Disabled))
        );

        match dispatch_type {
            EventDispatchType::Bubbling => {
                nodes.retain_mut(|node| node.borrow().element.is_some());

                // Sort by layout order descending.
                nodes.sort_unstable_by(|a_rc, b_rc| {
                    let a = a_rc.borrow();
                    let b = b_rc.borrow();
                    let a_elem = a.element.as_ref().unwrap();
                    let b_elem = b.element.as_ref().unwrap();

                    (
                        b.overlay_order,
                        b_elem.element_data().layout_item.layout_order,
                    )
                        .cmp(&(
                            a.overlay_order,
                            a_elem.element_data().layout_item.layout_order,
                        ))
                });

                // 1. Do a hit test to find the target element.
                // We order by the overlay depth descending and layout order descending.
                let mut target: Option<Rc<RefCell<FiberNode>>> = None;
                let mut targets: VecDeque<Rc<RefCell<FiberNode>>> = VecDeque::new();

                for node in nodes {
                    if let Some(element) = node.borrow().element {
                        let should_pass_hit_test =
                            mouse_position.is_some() && element.in_bounds(mouse_position.unwrap());

                        // The first element to pass the hit test should be the target.
                        if should_pass_hit_test && target.is_none() {
                            target = Some(Rc::clone(&node));
                        }

                        // Unless another element has pointer capture.
                        if let Some(element_id) = reactive_tree.pointer_captures.get(&DUMMY_DEVICE_ID)
                            && *element_id == element.component_id()
                            && (is_pointer_event || is_ime_event)
                        {
                            target = Some(Rc::clone(&node));
                            break;
                        }

                        if let Some(focus_id) = reactive_tree.focus
                            && is_keyboard_event
                            && element.component_id() == focus_id
                        {
                            target = Some(Rc::clone(&node));
                            break;
                        }
                    }
                }
                if target.is_none() {
                    return;
                }
                let target = target.unwrap();

                let mut current_target = Some(Rc::clone(&target));
                while let Some(node) = current_target {
                    targets.push_back(Rc::clone(&node));
                    current_target = node.borrow().parent.as_ref().and_then(|p| p.upgrade());
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

                    let mut current_component = Some(Rc::clone(&current_target));
                    while let Some(node) = current_component {
                        if node.borrow().element.is_none() {
                            closest_ancestor_component = Some(node.borrow().component);
                            break;
                        }
                        current_component = node.borrow().parent.as_ref().and_then(|p| p.upgrade());
                    }

                    // Dispatch the event to the element's component.
                    if let Some(node) = closest_ancestor_component {
                        let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
                        let mut event = Event::default();
                        let target_param = Some(target.borrow().element.unwrap());
                        let current_target_param = Some(current_target.borrow().element.unwrap());
                        (node.update)(
                            state,
                            global_state,
                            node.props.clone(),
                            &mut event,
                            message,
                            node.id,
                            window_context,
                            target_param,
                            current_target_param,
                        );

                        if !event.prevent_defaults
                            && event.propagate
                            && let Some(ref result_message) = event.result_message
                        {
                            element_events.push_back((result_message.clone(), target.borrow().element.unwrap()));
                        }

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
                    let target = Rc::clone(&targets[0]);
                    for current_target in targets.iter() {
                        if !propagate || prevent_defaults {
                            break;
                        }
                        if let Some(element) = current_target.borrow().element
                            && let Message::CraftMessage(event) = message
                        {
                            let mut res = Event::new();
                            let target_param = target.borrow().element;
                            let current_target_param = Some(element);
                            element.on_event(
                                event,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                is_style,
                                &mut res,
                                target_param,
                                current_target_param,
                            );
                            focus = focus.merge(res.focus);
                            reactive_tree.element_state.update_element_focus(res.focus);

                            if let Some(result_message) = res.result_message {
                                element_events.push_back((result_message, element));
                            }

                            propagate = propagate && res.propagate;
                            prevent_defaults = prevent_defaults || res.prevent_defaults;
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

                        let mut event = Event::default();

                        // Todo: are target and current_target correct?
                        let target_param = Some(*target_element);
                        let current_target_param = Some(*target_element);
                        if let Some(element) = current_target.element {
                            element.on_event(
                                message,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                // first_element && is_style. For only the first element.
                                is_style,
                                &mut event,
                                target_param,
                                current_target_param,
                            );
                            focus = focus.merge(event.focus);
                            reactive_tree.element_state.update_element_focus(event.focus);
                        } else {
                            let state = reactive_tree
                                .user_state
                                .storage
                                .get_mut(&current_target.component.id)
                                .unwrap()
                                .as_mut();
                            // For element events the target and current target
                            // are the element the event was dispatched from.
                            let target_param = Some(*target_element);
                            let current_target_param = Some(*target_element);
                            (current_target.component.update)(
                                state,
                                global_state,
                                current_target.component.props.clone(),
                                &mut event,
                                &Message::CraftMessage(message.clone()),
                                current_target.component.id,
                                window_context,
                                target_param,
                                current_target_param,
                            );
                        }
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
                    if node.borrow().component.id != id {
                        continue;
                    }

                    if let Some(element) = node.borrow().element {
                        if let Message::CraftMessage(message) = message {
                            let mut res = Event::new();
                            element.on_event(
                                message,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                false,
                                &mut res,
                                None,
                                None,
                            );
                            focus = focus.merge(res.focus);
                            reactive_tree.element_state.update_element_focus(res.focus);

                            effects.append(&mut res.effects);
                        }

                        break;
                    } else {
                        let component = node.borrow().component;
                        let state = reactive_tree.user_state.storage.get_mut(&component.id).unwrap().as_mut();
                        let mut event = Event::default();
                        (component.update)(
                            state,
                            global_state,
                            component.props.clone(),
                            &mut event,
                            message,
                            component.id,
                            window_context,
                            None,
                            None,
                        );
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
            EventDispatchType::Accesskit(_) => {}
            EventDispatchType::DirectToMatchingElements(user_by_predicate_fn) => {
                for node in nodes {
                    if let Some(element) = node.borrow().element {
                        if !user_by_predicate_fn(element) {
                            continue;
                        }

                        if let Message::CraftMessage(message) = message {
                            let mut res = Event::new();
                            element.on_event(
                                message,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                false,
                                &mut res,
                                None,
                                None,
                            );
                            focus = focus.merge(res.focus);
                            reactive_tree.element_state.update_element_focus(res.focus);

                            effects.append(&mut res.effects);
                        }
                    }
                }
            }
        }
    }
    reactive_tree.update_focus(focus);

    // Handle effects.
    for (dispatch_type, message) in effects.iter() {
        dispatch_event(
            message,
            dispatch_type.clone(),
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
