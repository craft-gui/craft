use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::app::ELEMENTS;
use crate::elements::ElementInternals;
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventKind};
use crate::text::text_context::TextContext;
use craft_renderer::RenderList;
use craft_renderer::renderer::TargetItem;
use kurbo::Point;

pub(super) fn freeze_target_list(
    target: Rc<RefCell<dyn ElementInternals>>,
) -> VecDeque<Rc<RefCell<dyn ElementInternals>>> {
    let mut current_target = Some(Rc::clone(&target));

    // Gather and "freeze" the elements we will visit.
    let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();
    while let Some(node) = current_target {
        targets.push_back(Rc::clone(&node));
        current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
    }

    targets
}

/// Find the target that should be visited.
pub(super) fn find_target(
    root: &Rc<RefCell<dyn ElementInternals>>,
    mouse_position: Option<Point>,
    message: &EventKind,
    render_list: &mut RenderList,
    target_scratch: &mut Vec<Rc<RefCell<dyn ElementInternals>>>,
    pointer_capture: &PointerCapture,
) -> Rc<RefCell<dyn ElementInternals>> {
    let mut target = pointer_capture.find_pointer_capture_target(message);
    if let Some(target) = target {
        return target;
    }

    ELEMENTS.with_borrow_mut(|elements| {
        TargetItem::sort_items_by_overlay_depth(&mut render_list.targets);
        target_scratch.extend(render_list.targets.iter().rev().filter_map(|target_item| {
            // When an element is removed from the dom, we do not remove it from targets.
            // So we must handle it here.
            elements.get(target_item.custom_id).and_then(|target| target.upgrade())
        }));
    });

    // Otherwise do hit-testing:

    for node in target_scratch.drain(..) {
        let should_pass_hit_test = mouse_position.is_some() && node.borrow().in_bounds(mouse_position.unwrap());

        // The first element to pass the hit test should be the target.
        if should_pass_hit_test && target.is_none() {
            target = Some(Rc::clone(&node));
            break;
        }
    }

    target.unwrap_or(Rc::clone(root))
}

pub(super) fn call_user_event_handlers(
    event: &mut Event,
    current_target: &Rc<RefCell<dyn ElementInternals>>,
    message: &EventKind,
) {
    match message {
        EventKind::PointerEnter() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_enter {
                (*handler)(event);
            }
        }
        EventKind::PointerLeave() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_leave {
                (*handler)(event);
            }
        }
        EventKind::PointerButtonUp(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_button_up {
                (*handler)(event, e);
            }
        }
        EventKind::PointerButtonDown(e) => {
            let len = current_target.borrow().element_data().on_pointer_button_down.len();
            for i in 0..len {
                let handler = current_target.borrow().element_data().on_pointer_button_down[i].clone();
                (*handler)(event, e);
            }
        }
        EventKind::KeyboardInputEvent(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_keyboard_input {
                (*handler)(event, e);
            }
        }
        EventKind::PointerMovedEvent(e) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_pointer_moved {
                (*handler)(event, e);
            }
        }
        EventKind::PointerScroll(_) => {}
        EventKind::ImeEvent(_) => {}
        EventKind::TextInputChanged(_) => {}
        EventKind::LinkClicked(_) => {}
        EventKind::DropdownToggled(_) => {}
        EventKind::DropdownItemSelected(item) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_dropdown_item_selected {
                (*handler)(event, *item);
            }
        }
        EventKind::SwitchToggled(_) => {}
        EventKind::SliderValueChanged(slider_value) => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_slider_value_changed {
                (*handler)(event, *slider_value);
            }
        }
        EventKind::ElementMessage(_) => {}
        EventKind::GotPointerCapture() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_got_pointer_capture {
                (*handler)(event);
            }
        }
        EventKind::LostPointerCapture() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_lost_pointer_capture {
                (*handler)(event);
            }
        }
        EventKind::Scroll() => {
            let element_data = current_target.borrow().element_data().clone();

            for handler in &element_data.on_scroll {
                (*handler)(event);
            }
        }
    }
}

pub(super) fn call_default_element_event_handler(
    event: &mut Event,
    current_target: &Rc<RefCell<dyn ElementInternals>>,
    target: &Rc<RefCell<dyn ElementInternals>>,
    text_context: &mut Option<TextContext>,
    message: &EventKind,
) {
    current_target
        .borrow_mut()
        .on_event(message, text_context.as_mut().unwrap(), event, Some(target.clone()));
}
