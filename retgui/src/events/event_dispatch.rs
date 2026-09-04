use std::collections::{HashMap, VecDeque};

use retgui_primitives::geometry::Point;
use retgui_renderer::renderer::Renderer;

use crate::elements::{DynElement, Elements, WindowNode};
use crate::events::helpers::{TargetSearchContext, call_user_event_handlers, find_target, freeze_target_list, nearest_common_ancestor};
use crate::events::{ClickEvent, ClickTrigger, Event, EventKind, PointerButton, PointerEnterEvent, PointerId, PointerLeaveEvent};
use crate::text::text_context::TextContext;

pub(super) fn dispatch_event(
    event: &mut EventKind,
    targets: &VecDeque<DynElement>,
    text_context: &mut TextContext,
    elements: &mut Elements,
) {
    for target in targets.iter().rev() {
        event.base_mut().current_target = *target;
        call_user_event_handlers(event, true, elements);
        if event.is_propagation_stopped() {
            break;
        }
    }

    if !event.is_propagation_stopped() {
        for target in targets {
            event.base_mut().current_target = *target;
            call_user_event_handlers(event, false, elements);
            if event.is_propagation_stopped() {
                break;
            }
        }
    }

    if !event.is_default_prevented() {
        for target in targets {
            event.base_mut().current_target = *target;
            elements.try_dispatch_mut(*target, |target, elements| {
                target.on_event(elements, event, text_context)
            });
            if event.is_propagation_stopped() {
                break;
            }
        }
    }
}

pub(super) fn dispatch_event_once(event: &mut EventKind, text_context: &mut TextContext, elements: &mut Elements) {
    call_user_event_handlers(event, true, elements);
    if !event.is_propagation_stopped() {
        call_user_event_handlers(event, false, elements);
    }
    if !event.is_default_prevented() {
        let target = event.target();
        elements.try_dispatch_mut(target, |target, elements| {
            target.on_event(elements, event, text_context)
        });
    }
}

pub(crate) struct EventDispatcher {
    previous_targets: VecDeque<DynElement>,
    active_pointer_targets: HashMap<PointerId, DynElement>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            previous_targets: VecDeque::new(),
            active_pointer_targets: HashMap::new(),
        }
    }

    pub(crate) fn dispatch_queued_events(&mut self, text_context: &mut TextContext, elements: &mut Elements) {
        while let Some(mut event) = elements.dequeue_event() {
            if !elements.contains(event.target()) {
                continue;
            }
            let targets = freeze_target_list(event.target(), elements);
            dispatch_event(&mut event, &targets, text_context, elements);
        }
    }

    fn maybe_dispatch_pointer_leave(
        &self,
        text_context: &mut TextContext,
        targets: &VecDeque<DynElement>,
        elements: &mut Elements,
    ) {
        for previous in self.previous_targets.iter().copied() {
            if elements.contains(previous) && !targets.contains(&previous) {
                let mut event = EventKind::PointerLeave(PointerLeaveEvent::new(previous));
                dispatch_event_once(&mut event, text_context, elements);
            }
        }
    }

    fn maybe_dispatch_pointer_enter(
        &self,
        text_context: &mut TextContext,
        targets: &VecDeque<DynElement>,
        elements: &mut Elements,
    ) {
        for target in targets.iter().rev().copied() {
            if !self.previous_targets.contains(&target) {
                let mut event = EventKind::PointerEnter(PointerEnterEvent::new(target));
                dispatch_event_once(&mut event, text_context, elements);
            }
        }
    }

    fn maybe_dispatch_pointer_click(
        &mut self,
        dispatched_target: Option<DynElement>,
        captured: bool,
        event_kind: &EventKind,
        text_context: &mut TextContext,
        elements: &mut Elements,
    ) {
        match event_kind {
            EventKind::PointerDown(event)
                if event.pointer.is_primary_pointer() && event.button == Some(PointerButton::Left) =>
            {
                if let (Some(pointer), Some(target)) = (event.pointer.pointer_id, dispatched_target) {
                    self.active_pointer_targets.insert(pointer, target);
                }
            }
            EventKind::PointerUp(event)
                if event.pointer.is_primary_pointer() && event.button == Some(PointerButton::Left) =>
            {
                let pointer = event.pointer.pointer_id.unwrap();
                if let (Some(down), Some(up)) = (self.active_pointer_targets.get(&pointer).copied(), dispatched_target)
                {
                    let target = if captured {
                        Some(up)
                    } else {
                        nearest_common_ancestor(down, up, elements)
                    };
                    if let Some(target) = target {
                        let trigger = ClickTrigger::Pointer {
                            button: event.button,
                            position: event.position,
                        };
                        let mut click = EventKind::Click(ClickEvent::new(target, trigger));
                        let targets = freeze_target_list(target, elements);
                        dispatch_event(&mut click, &targets, text_context, elements);
                    }
                }
                self.active_pointer_targets.remove(&pointer);
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_event(
        &mut self,
        event_kind: &mut EventKind,
        mouse_position: Option<Point>,
        root: DynElement,
        text_context: &mut TextContext,
        renderer: &mut dyn Renderer,
        target_scratch: &mut Vec<DynElement>,
        elements: &mut Elements,
    ) -> bool {
        let window = elements.get(root).element_data().window.unwrap_or(root);
        let mut targets = VecDeque::new();
        let mut captured = false;

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = event_kind.pointer_id()
        {
            let capture = &elements.get_as::<WindowNode>(window).pointer_capture;
            captured = capture.find_pointer_capture_target(event_kind, &pointer_id).is_some();
            let target = find_target(
                root,
                mouse_position,
                event_kind,
                &pointer_id,
                TargetSearchContext {
                    renderer,
                    target_scratch,
                    pointer_capture: capture,
                    elements,
                },
            );
            targets = freeze_target_list(target, elements);
        } else if event_kind.is_keyboard_event()
            && let Some(focus) = elements.focus
            && elements.contains(focus)
        {
            targets = freeze_target_list(focus, elements);
        }

        if targets.is_empty() {
            targets.push_back(root);
        }
        if event_kind.is_system_pointer_event() {
            self.maybe_dispatch_pointer_leave(text_context, &targets, elements);
            self.maybe_dispatch_pointer_enter(text_context, &targets, elements);
        }

        event_kind.retarget(targets[0]);
        dispatch_event(event_kind, &targets, text_context, elements);
        let prevented = event_kind.is_default_prevented();
        let dispatched_target =
            matches!(event_kind, EventKind::PointerUp(_) | EventKind::PointerDown(_)).then(|| event_kind.target());

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = event_kind.pointer_id()
        {
            let mut capture = std::mem::take(&mut elements.get_as_mut::<WindowNode>(window).pointer_capture);
            let changed =
                capture.maybe_handle_implicit_pointer_capture_release(elements, event_kind, text_context, &pointer_id);
            let during_dispatch = std::mem::take(&mut elements.get_as_mut::<WindowNode>(window).pointer_capture);
            capture
                .pending_pointer_captures
                .extend(during_dispatch.pending_pointer_captures);
            elements.get_as_mut::<WindowNode>(window).pointer_capture = capture;
            if changed {
                let capture = &elements.get_as::<WindowNode>(window).pointer_capture;
                let target = find_target(
                    root,
                    mouse_position,
                    event_kind,
                    &pointer_id,
                    TargetSearchContext {
                        renderer,
                        target_scratch,
                        pointer_capture: capture,
                        elements,
                    },
                );
                targets = freeze_target_list(target, elements);
                self.maybe_dispatch_pointer_leave(text_context, &targets, elements);
                self.maybe_dispatch_pointer_enter(text_context, &targets, elements);
            }
        }

        self.maybe_dispatch_pointer_click(dispatched_target, captured, event_kind, text_context, elements);
        if event_kind.is_system_pointer_event() {
            self.previous_targets = targets;
        }
        self.dispatch_queued_events(text_context, elements);
        prevented
    }
}
