use crate::elements::Element;
use crate::events::pointer_capture::find_pointer_capture_target;
use crate::events::{CraftMessage, Event};
use crate::text::text_context::TextContext;
use kurbo::Point;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use crate::app::{SPATIAL_TREE};

pub(super) fn freeze_target_list(target: Rc<RefCell<dyn Element>>) -> VecDeque<Rc<RefCell<dyn Element>>> {
    let mut current_target = Some(Rc::clone(&target));

    // Gather and "freeze" the elements we will visit.
    let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();
    while let Some(node) = current_target {
        targets.push_back(Rc::clone(&node));
        current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
    }

    targets
}

/// Find the target that should be visited.
/// Find the target that should be visited.
pub(super) fn find_target(
    root: &Rc<RefCell<dyn Element>>,
    mouse_position: Option<Point>,
    message: &CraftMessage,
) -> Rc<RefCell<dyn Element>> {
    let target = find_pointer_capture_target(message);
    if let Some(target) = target {
        return target;
    }

    // Otherwise do hit-testing:

    let target: Option<Rc<RefCell<dyn Element>>> = SPATIAL_TREE.with_borrow_mut(|spatial_tree| {
        if mouse_position.is_none() {
            return None;
        };
        let target = spatial_tree.hit_test_point(mouse_position.unwrap());
        target.and_then(|target| target.upgrade())
    });

    target.unwrap_or(Rc::clone(root))
}

pub(super) fn call_user_event_handlers(
    event: &mut Event,
    current_target: &Rc<RefCell<dyn Element>>,
    message: &CraftMessage,
) {
    match message {
        CraftMessage::PointerEnter() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_enter {
                (*handler)(event);
            }
        }
        CraftMessage::PointerLeave() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_leave {
                (*handler)(event);
            }
        }
        CraftMessage::PointerButtonUp(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_button_up {
                (*handler)(event, e);
            }
        }
        CraftMessage::PointerButtonDown(e) => {
            let len = current_target.borrow().element_data().on_pointer_button_down.len();
            for i in 0..len {
                let handler = current_target.borrow().element_data().on_pointer_button_down[i].clone();
                (*handler)(event, e);
            }
        }
        CraftMessage::KeyboardInputEvent(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_keyboard_input {
                (*handler)(event, e);
            }
        }
        CraftMessage::PointerMovedEvent(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_moved {
                (*handler)(event, e);
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
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_got_pointer_capture {
                (*handler)(event);
            }
        }
        CraftMessage::LostPointerCapture() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_lost_pointer_capture {
                (*handler)(event);
            }
        }
    }
}

pub(super) fn call_default_element_event_handler(
    event: &mut Event,
    current_target: &Rc<RefCell<dyn Element>>,
    target: &Rc<RefCell<dyn Element>>,
    text_context: &mut Option<TextContext>,
    message: &CraftMessage,
) {
    current_target.borrow_mut().on_event(message, text_context.as_mut().unwrap(), event, Some(target.clone()));
}
