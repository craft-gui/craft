use retgui_primitives::geometry::Point;
use retgui_renderer::renderer::Renderer;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use ui_events::pointer::PointerId;

use retgui_renderer::TargetItem;

use crate::app::ELEMENTS;
use crate::elements::ElementInternals;
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventCallback, EventCallbackKind, EventKind};

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

pub(super) fn nearest_common_ancestor(
    a: &Rc<RefCell<dyn ElementInternals>>,
    b: &Rc<RefCell<dyn ElementInternals>>,
) -> Option<Rc<RefCell<dyn ElementInternals>>> {
    let a_targets = freeze_target_list(a.clone());
    let b_targets = freeze_target_list(b.clone());

    for b_target in b_targets {
        let b_id = b_target.borrow().id();

        if a_targets.iter().any(|a_target| a_target.borrow().id() == b_id) {
            return Some(b_target);
        }
    }

    None
}

/// Find the target that should be visited.
pub(super) fn find_target(
    root: &Rc<RefCell<dyn ElementInternals>>,
    mouse_position: Option<Point>,
    message: &EventKind,
    renderer: &mut dyn Renderer,
    target_scratch: &mut Vec<Rc<RefCell<dyn ElementInternals>>>,
    pointer_capture: &PointerCapture,
    pointer_id: &PointerId,
) -> Rc<RefCell<dyn ElementInternals>> {
    let mut target = pointer_capture.find_pointer_capture_target(message, pointer_id);
    if let Some(target) = target {
        return target;
    }

    let physical_mouse_position = mouse_position.map(|point| {
        let scale_factor = root.borrow().element_data().applied_scale_factor;
        Point::new(point.x * scale_factor, point.y * scale_factor)
    });

    ELEMENTS.with_borrow_mut(|elements| {
        let targets = &mut renderer.render_list_mut().targets;
        TargetItem::sort_items_by_overlay_depth(targets);
        target_scratch.extend(targets.iter().rev().filter_map(|target_item| {
            if !physical_mouse_position.is_some_and(|point| target_item.rectangle.contains(&point)) {
                return None;
            }
            // When an element is removed from the dom, we do not remove it from targets.
            // So we must handle it here.
            elements.get(target_item.custom_id).and_then(|target| target.upgrade())
        }));
    });

    // Otherwise do hit-testing:

    for node in target_scratch.drain(..) {
        let should_pass_hit_test = mouse_position.is_some_and(|point| node.borrow().in_bounds(point));

        // The first element to pass the hit test should be the target.
        if should_pass_hit_test && target.is_none() {
            target = Some(Rc::clone(&node));
            break;
        }
    }

    target.unwrap_or(Rc::clone(root))
}

pub(super) fn call_user_event_handlers(event: &mut Event, message: &EventKind, capturing: bool) {
    let current_target = event.current_target.clone();
    let callbacks = current_target.borrow().element_data().event_callbacks.clone();

    for EventCallback {
        callback,
        capturing: callback_capturing,
    } in callbacks
    {
        if callback_capturing != capturing {
            continue;
        }

        match (message, callback) {
            (EventKind::PointerEnter(), EventCallbackKind::PointerEnter(handler))
            | (EventKind::PointerLeave(), EventCallbackKind::PointerLeave(handler))
            | (EventKind::Click(), EventCallbackKind::Click(handler))
            | (EventKind::GotPointerCapture(), EventCallbackKind::GotPointerCapture(handler))
            | (EventKind::LostPointerCapture(), EventCallbackKind::LostPointerCapture(handler))
            | (EventKind::Scroll(), EventCallbackKind::Scroll(handler)) => {
                handler(event);
            }
            (EventKind::PointerButtonUp(pointer_event), EventCallbackKind::PointerButtonUp(handler))
            | (EventKind::PointerButtonDown(pointer_event), EventCallbackKind::PointerButtonDown(handler)) => {
                handler(event, pointer_event);
            }
            (EventKind::KeyboardInputEvent(keyboard_event), EventCallbackKind::KeyboardInput(handler)) => {
                handler(event, keyboard_event);
            }
            (EventKind::PointerMovedEvent(pointer_update), EventCallbackKind::PointerMoved(handler)) => {
                handler(event, pointer_update);
            }
            (EventKind::DropdownItemSelected(item), EventCallbackKind::DropdownItemSelected(handler)) => {
                handler(event, *item);
            }
            (EventKind::SliderValueChanged(value), EventCallbackKind::SliderValueChanged(handler)) => {
                handler(event, *value);
            }
            (EventKind::RadioValueChanged(value), EventCallbackKind::RadioValueChanged(handler)) => {
                handler(event, value.clone());
            }
            (EventKind::CheckboxToggled(value), EventCallbackKind::CheckboxToggled(handler)) => {
                handler(event, value.clone());
            }
            (EventKind::TextInputChanged(value), EventCallbackKind::TextInputChanged(handler)) => {
                handler(event, value);
            }
            _ => {}
        }
    }
}
