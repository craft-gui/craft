use std::collections::VecDeque;

use retgui_primitives::geometry::Point;
use retgui_renderer::TargetItem;
use retgui_renderer::renderer::Renderer;

use crate::elements::{DynElement, Elements};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventCallback, EventCallbackKind, EventKind, PointerId};

pub(super) fn freeze_target_list(target: DynElement, elements: &Elements) -> VecDeque<DynElement> {
    let mut current = Some(target);
    let mut targets = VecDeque::new();
    while let Some(node) = current {
        targets.push_back(node);
        current = elements.get(node).parent();
    }
    targets
}

pub(super) fn nearest_common_ancestor(a: DynElement, b: DynElement, elements: &Elements) -> Option<DynElement> {
    let a_targets = freeze_target_list(a, elements);
    freeze_target_list(b, elements)
        .into_iter()
        .find(|candidate| a_targets.contains(candidate))
}

pub(super) struct TargetSearchContext<'a> {
    pub renderer: &'a mut dyn Renderer,
    pub target_scratch: &'a mut Vec<DynElement>,
    pub pointer_capture: &'a PointerCapture,
    pub elements: &'a Elements,
}

pub(super) fn find_target(
    root: DynElement,
    mouse_position: Option<Point>,
    message: &EventKind,
    pointer_id: &PointerId,
    context: TargetSearchContext<'_>,
) -> DynElement {
    let TargetSearchContext {
        renderer,
        target_scratch,
        pointer_capture,
        elements,
    } = context;

    if let Some(target) = pointer_capture.find_pointer_capture_target(message, pointer_id) {
        return target;
    }

    let physical_mouse_position = mouse_position.map(|point| {
        let scale = elements.get(root).element_data().applied_scale_factor;
        Point::new(point.x * scale, point.y * scale)
    });

    let targets = &mut renderer.render_list_mut().targets;
    TargetItem::sort_items_by_overlay_depth(targets);
    target_scratch.extend(targets.iter().rev().filter_map(|item| {
        physical_mouse_position
            .is_some_and(|point| item.rectangle.contains(&point))
            .then(|| elements.by_internal_id(item.custom_id))
            .flatten()
    }));

    target_scratch
        .drain(..)
        .find(|node| mouse_position.is_some_and(|point| elements.get(*node).in_bounds(point)))
        .unwrap_or(root)
}

pub(super) fn call_user_event_handlers(event: &mut EventKind, capturing: bool, elements: &mut Elements) {
    let Some(current_target) = elements.try_get(event.current_target()) else {
        return;
    };
    let callbacks = current_target.element_data().event_callbacks.clone();

    for EventCallback {
        callback,
        capturing: callback_capturing,
    } in callbacks
    {
        if callback_capturing != capturing {
            continue;
        }
        match (&mut *event, callback) {
            (EventKind::PointerEnter(event), EventCallbackKind::PointerEnter(handler)) => handler(event, elements),
            (EventKind::PointerLeave(event), EventCallbackKind::PointerLeave(handler)) => handler(event, elements),
            (EventKind::Click(event), EventCallbackKind::Click(handler)) => handler(event, elements),
            (EventKind::Custom(event), EventCallbackKind::Custom(handler)) => handler(event, elements),
            (EventKind::Focus(event), EventCallbackKind::Focus(handler)) => handler(event, elements),
            (EventKind::GotPointerCapture(event), EventCallbackKind::GotPointerCapture(handler))
            | (EventKind::LostPointerCapture(event), EventCallbackKind::LostPointerCapture(handler)) => {
                handler(event, elements)
            }
            (EventKind::Scroll(event), EventCallbackKind::Scroll(handler)) => handler(event, elements),
            (EventKind::Unfocus(event), EventCallbackKind::Unfocus(handler)) => handler(event, elements),
            (EventKind::PointerUp(event), EventCallbackKind::PointerButtonUp(handler))
            | (EventKind::PointerDown(event), EventCallbackKind::PointerButtonDown(handler)) => {
                handler(event, elements)
            }
            (EventKind::KeyDown(event), EventCallbackKind::KeyboardInput(handler))
            | (EventKind::KeyUp(event), EventCallbackKind::KeyboardInput(handler)) => handler(event, elements),
            (EventKind::PointerMoved(event), EventCallbackKind::PointerMoved(handler)) => handler(event, elements),
            (EventKind::DropdownItemSelected(event), EventCallbackKind::DropdownItemSelected(handler)) => {
                handler(event, elements)
            }
            (EventKind::SliderValueChanged(event), EventCallbackKind::SliderValueChanged(handler)) => {
                handler(event, elements)
            }
            (EventKind::RadioValueChanged(event), EventCallbackKind::RadioValueChanged(handler)) => {
                handler(event, elements)
            }
            (EventKind::CheckboxToggled(event), EventCallbackKind::CheckboxToggled(handler)) => {
                handler(event, elements)
            }
            (EventKind::TextInputChanged(event), EventCallbackKind::TextInputChanged(handler)) => {
                handler(event, elements)
            }
            _ => {}
        }
    }
}
