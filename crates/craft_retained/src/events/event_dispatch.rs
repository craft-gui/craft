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

#[allow(clippy::too_many_arguments)]
pub fn dispatch_event(
    message: &CraftMessage,
    dispatch_type: EventDispatchType,
    _resource_manager: &mut Arc<ResourceManager>,
    mouse_position: Option<Point>,
    root: Rc<RefCell<dyn Element>>,
    text_context: &mut Option<TextContext>,
    window_context: &mut WindowContext,
    is_style: bool,
) {
    let mut focus = FocusAction::None;
    let span = span!(Level::INFO, "dispatch event");
    let _enter = span.enter();

    let mut effects: Vec<(EventDispatchType, CraftMessage)> = Vec::new();

    {
/*        let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
            current_element_tree
        } else {
            return;
        };

        let fiber: Rc<RefCell<FiberNode>> =
            fiber_tree::new(reactive_tree.component_tree.as_ref().unwrap(), current_element_tree.as_ref());*/

        let mut nodes: Vec<Rc<RefCell<dyn Element>>> = Vec::new();
        let mut to_visit: Vec<Rc<RefCell<dyn Element>>> = vec![Rc::clone(&root)];

        while let Some(node_rc) = to_visit.pop() {
            let node_ref = node_rc.borrow();

            nodes.push(Rc::clone(&node_rc));

            for child in node_ref.children().iter().rev() {
                to_visit.push(Rc::clone(child));
            }
        }

        let is_pointer_event = matches!(
            message,
            CraftMessage::PointerMovedEvent(_)
                | CraftMessage::PointerButtonUp(_)
                | CraftMessage::PointerButtonDown(_)
        );
        let is_keyboard_event = matches!(message, CraftMessage::KeyboardInputEvent(_));
        let is_ime_event = matches!(
            message,
            CraftMessage::ImeEvent(Ime::Enabled)
                | CraftMessage::ImeEvent(Ime::Disabled)
        );

        match dispatch_type {
            EventDispatchType::Bubbling => {
                // Sort by layout order descending.
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

                // 1. Do a hit test to find the target element.
                // We order by the overlay depth descending and layout order descending.
                let mut target: Option<Rc<RefCell<dyn Element>>> = None;
                let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();

                for node in nodes {
                        let should_pass_hit_test =
                            mouse_position.is_some() && node.borrow().in_bounds(mouse_position.unwrap());

                        // The first element to pass the hit test should be the target.
                        if should_pass_hit_test && target.is_none() {
                            target = Some(Rc::clone(&node));
                        }

                        let pointer_capture_element_id = DOCUMENTS.with_borrow_mut(|docs| {
                            let key = &PointerId::new(1).unwrap();
                            docs.get_current_document().pointer_captures.get(key).map(|id| *id)
                        });

                        // Unless another element has pointer capture.
                        if let Some(element_id) = pointer_capture_element_id
                            && element_id == node.borrow().id()
                            && (is_pointer_event || is_ime_event)
                        {
                            target = Some(Rc::clone(&node));
                            break;
                        }

                        /*if let Some(focus_id) = reactive_tree.focus
                            && is_keyboard_event
                            && element.component_id() == focus_id
                        {
                            target = Some(Rc::clone(&node));
                            break;
                        }*/
                }
                if target.is_none() {
                    return;
                }
                let target = target.unwrap();

                let mut current_target = Some(Rc::clone(&target));
                while let Some(node) = current_target {
                    targets.push_back(Rc::clone(&node));
                    current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
                }

                if targets.is_empty() {
                    return;
                }

                let target = targets[0].clone();
                let mut propagate = true;
                let mut prevent_defaults = false;


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
                        }
                }

                for current_target in targets.iter() {
                    let mut res = Event::new();
                    current_target.borrow_mut().on_event(message, text_context.as_mut().unwrap(), false, &mut res, Some(target.clone()));
                    if !propagate {
                        break;
                    }
                }

               /* for element_state in reactive_tree.element_state.storage.values_mut() {
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
                }*/
            }
        }
    }
    //reactive_tree.update_focus(focus);
}
