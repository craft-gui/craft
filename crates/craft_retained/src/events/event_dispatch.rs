use crate::elements::Element;
use crate::events::{CraftMessage, Event, EventDispatchType, FocusAction};

use crate::text::text_context::TextContext;
use crate::window_context::WindowContext;
use craft_logging::{span, Level};
use craft_primitives::geometry::Point;
use craft_resource_manager::ResourceManager;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use ui_events::pointer::PointerId;
use winit::event::Ime;
use crate::app::DOCUMENTS;
use crate::events::pointer_capture_dispatch::{find_pointer_capture_target, processing_pending_pointer_capture};

/// Collect all the elements into an array.
pub fn collect_nodes(root: &Rc<RefCell<dyn Element>>) -> Vec<Rc<RefCell<dyn Element>>> {
    let mut nodes: Vec<Rc<RefCell<dyn Element>>> = Vec::new();
    let mut to_visit: Vec<Rc<RefCell<dyn Element>>> = vec![Rc::clone(root)];
    while let Some(node_rc) = to_visit.pop() {
        let node_ref = node_rc.borrow();

        nodes.push(Rc::clone(&node_rc));

        for child in node_ref.children().iter().rev() {
            to_visit.push(Rc::clone(child));
        }
    }

    nodes
}

/// Find the target that should be visited.
pub fn find_target(root: &Rc<RefCell<dyn Element>>, mouse_position: Option<Point>, message: &CraftMessage) -> Rc<RefCell<dyn Element>> {
    let mut nodes: Vec<Rc<RefCell<dyn Element>>> = collect_nodes(root);

    let mut target = find_pointer_capture_target(&nodes, message);
    if let Some(target) = target {
        return target;
    }

    // Otherwise sort and do hit-testing:

    // Sort by layout order in descending order.
    nodes.sort_unstable_by(|a_rc, b_rc| {
        let a = a_rc.borrow();
        let b = b_rc.borrow();
        let a_elem = a;
        let b_elem = b;

        (
            1, //b.overlay_order,
            b_elem.element_data().layout_item.layout_order,
        )
            .cmp(&(
                1, //a.overlay_order,
                a_elem.element_data().layout_item.layout_order,
            ))
    });


    for node in nodes {
        let should_pass_hit_test =
            mouse_position.is_some() && node.borrow().in_bounds(mouse_position.unwrap());

        // The first element to pass the hit test should be the target.
        if should_pass_hit_test && target.is_none() {
            target = Some(Rc::clone(&node));
        }
    }

    target.unwrap_or(Rc::clone(root))
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_event(
    message: &CraftMessage,
    dispatch_type: EventDispatchType,
    resource_manager: &mut Arc<ResourceManager>,
    mouse_position: Option<Point>,
    root: Rc<RefCell<dyn Element>>,
    text_context: &mut Option<TextContext>,
    window_context: &mut WindowContext,
    is_style: bool,
) {
    let mut _focus = FocusAction::None;
    let span = span!(Level::INFO, "dispatch event");
    let _enter = span.enter();

    let is_pointer_up_event = matches!(message, CraftMessage::PointerButtonUp(_));
    let _is_keyboard_event = matches!(message, CraftMessage::KeyboardInputEvent(_));
    let _is_ime_event = matches!(
            message,
            CraftMessage::ImeEvent(Ime::Enabled)
                | CraftMessage::ImeEvent(Ime::Disabled)
        );

    {

        match dispatch_type {
            EventDispatchType::Bubbling => {

                let target: Rc<RefCell<dyn Element>> = find_target(&root, mouse_position, message);
                let mut current_target = Some(Rc::clone(&target));

                // Gather the elements to visit during the bubble phase.
                let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();
                while let Some(node) = current_target {
                    targets.push_back(Rc::clone(&node));
                    current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
                }

                let target = targets[0].clone();
                let propagate = true;


                // Call the callback handlers.
                for current_target in targets.iter() {

                    let mut res = Event::new();

                        let element_data = current_target.borrow();
                        let element_data = element_data.element_data();
                        match message {
                            CraftMessage::PointerButtonUp(e) => {
                                for handler in &element_data.on_pointer_button_up {
                                    (*handler)(&mut res, e);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                            CraftMessage::PointerButtonDown(e) => {
                                for handler in &element_data.on_pointer_button_down {
                                    (*handler)(&mut res, e);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                            CraftMessage::KeyboardInputEvent(e) => {
                                for handler in &element_data.on_keyboard_input {
                                    (*handler)(&mut res, e);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                            CraftMessage::PointerMovedEvent(e) => {
                                for handler in &element_data.on_pointer_moved {
                                    (*handler)(&mut res, e);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                            CraftMessage::PointerScroll(_) => {}
                            CraftMessage::ImeEvent(_) => {}
                            CraftMessage::TextInputChanged(_) => {}
                            CraftMessage::LinkClicked(_) => {}
                            CraftMessage::DropdownToggled(_) => {}
                            CraftMessage::DropdownItemSelected(_) => {}
                            CraftMessage::SwitchToggled(_) => {}
                            CraftMessage::SliderValueChanged(_) => {}
                            CraftMessage::ElementMessage(_) => {}
                            CraftMessage::GotPointerCapture() => {
                                for handler in &element_data.on_got_pointer_capture {
                                    (*handler)(&mut res);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                            CraftMessage::LostPointerCapture() => {
                                for handler in &element_data.on_lost_pointer_capture {
                                    (*handler)(&mut res);
                                    if !propagate {
                                        break;
                                    }
                                }
                            }
                        }
                }

                // Call the default on_event element functions.
                for current_target in targets.iter() {
                    let mut res = Event::new();
                    current_target.borrow_mut().on_event(message, text_context.as_mut().unwrap(), &mut res, Some(target.clone()));
                    if !propagate {
                        break;
                    }
                }
                

                // 9.5 Implicit release of pointer capture
                // https://w3c.github.io/pointerevents/#implicit-release-of-pointer-capture
                if is_pointer_up_event /* || is_pointer_canceled */ {
                    // Immediately after firing the pointerup or pointercancel events, the user agent MUST clear the pending pointer capture target override
                    // for the pointerId of the pointerup or pointercancel event that was just dispatched
                    DOCUMENTS.with_borrow_mut(|docs| {
                        let key = &PointerId::new(1).unwrap();
                        let _ = docs.get_current_document().pending_pointer_captures.remove(key);
                    });

                    processing_pending_pointer_capture(dispatch_type, resource_manager, mouse_position, root, text_context, window_context, is_style);
                } else if message.is_pointer_event() && !message.is_got_or_lost_pointer_capture() {
                    processing_pending_pointer_capture(dispatch_type, resource_manager, mouse_position, root, text_context, window_context, is_style);
                }

            }
        }
    }
}
