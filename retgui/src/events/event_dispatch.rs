use std::collections::{HashMap, VecDeque};

use retgui_primitives::geometry::Point;

use crate::App;
use crate::elements::{DynElement, WindowElement};
use crate::events::helpers::{TargetSearchContext, call_user_event_handlers, find_target, freeze_target_list, nearest_common_ancestor};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{ClickEvent, ClickTrigger, Event, EventKind, PointerButton, PointerEnterEvent, PointerId, PointerLeaveEvent};

pub(super) fn dispatch_event(event: &mut EventKind, targets: &VecDeque<DynElement>, app: &mut App) {
    for target in targets.iter().rev() {
        event.base_mut().current_target = *target;
        call_user_event_handlers(event, true, app);
        if event.is_propagation_stopped() {
            break;
        }
    }

    if !event.is_propagation_stopped() {
        for target in targets {
            event.base_mut().current_target = *target;
            call_user_event_handlers(event, false, app);
            if event.is_propagation_stopped() {
                break;
            }
        }
    }

    if !event.is_default_prevented() {
        for target in targets {
            event.base_mut().current_target = *target;
            app.elements.try_dispatch_mut(*target, |target, retained_elements| {
                target.on_event(
                    retained_elements,
                    &mut app.gummy_tree,
                    &app.access_tree,
                    &mut app.by_internal_id,
                    &mut app.event_queue,
                    &mut app.focus,
                    app.focus_outline_visible,
                    &mut app.pending_animation_updates,
                    &mut app.states,
                    event,
                    &mut app.text_context,
                );
            });
            if event.is_propagation_stopped() {
                break;
            }
        }
    }
}

pub(super) fn dispatch_event_once(event: &mut EventKind, app: &mut App) {
    call_user_event_handlers(event, true, app);
    if !event.is_propagation_stopped() {
        call_user_event_handlers(event, false, app);
    }
    if !event.is_default_prevented() {
        let target = event.target();
        app.elements.try_dispatch_mut(target, |target, retained_elements| {
            target.on_event(
                retained_elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                &mut app.event_queue,
                &mut app.focus,
                app.focus_outline_visible,
                &mut app.pending_animation_updates,
                &mut app.states,
                event,
                &mut app.text_context,
            );
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

    pub(crate) fn dispatch_queued_events(app: &mut App) {
        while let Some(mut event) = app.event_queue.pop_front() {
            if !app.elements.contains(event.target()) {
                continue;
            }
            let targets = freeze_target_list(event.target(), &app.elements);
            dispatch_event(&mut event, &targets, app);
        }
    }

    fn maybe_dispatch_pointer_leave(targets: &VecDeque<DynElement>, app: &mut App) {
        for previous in app.event_dispatcher.previous_targets.clone() {
            if app.elements.contains(previous) && !targets.contains(&previous) {
                let mut event = EventKind::PointerLeave(PointerLeaveEvent::new(previous));
                dispatch_event_once(&mut event, app);
            }
        }
    }

    fn maybe_dispatch_pointer_enter(targets: &VecDeque<DynElement>, app: &mut App) {
        for target in targets.iter().rev().copied() {
            if !app.event_dispatcher.previous_targets.contains(&target) {
                let mut event = EventKind::PointerEnter(PointerEnterEvent::new(target));
                dispatch_event_once(&mut event, app);
            }
        }
    }

    fn maybe_dispatch_pointer_click(
        dispatched_target: Option<DynElement>,
        captured: bool,
        event_kind: &EventKind,
        app: &mut App,
    ) {
        match event_kind {
            EventKind::PointerDown(event)
                if event.pointer.is_primary_pointer() && event.button == Some(PointerButton::Left) =>
            {
                if let (Some(pointer), Some(target)) = (event.pointer.pointer_id, dispatched_target) {
                    app.event_dispatcher.active_pointer_targets.insert(pointer, target);
                }
            }
            EventKind::PointerUp(event)
                if event.pointer.is_primary_pointer() && event.button == Some(PointerButton::Left) =>
            {
                let pointer = event.pointer.pointer_id.unwrap();
                if let (Some(down), Some(up)) = (
                    app.event_dispatcher.active_pointer_targets.get(&pointer).copied(),
                    dispatched_target,
                ) {
                    let target = if captured {
                        Some(up)
                    } else {
                        nearest_common_ancestor(down, up, &app.elements)
                    };
                    if let Some(target) = target {
                        let trigger = ClickTrigger::Pointer {
                            button: event.button,
                            position: event.position,
                        };
                        let mut click = EventKind::Click(ClickEvent::new(target, trigger));
                        let targets = freeze_target_list(target, &app.elements);
                        dispatch_event(&mut click, &targets, app);
                    }
                }
                app.event_dispatcher.active_pointer_targets.remove(&pointer);
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_event(
        event_kind: &mut EventKind,
        mouse_position: Option<Point>,
        root: DynElement,
        app: &mut App,
    ) -> bool {
        let window = app.elements.get(root).element_data().window.unwrap_or(root);
        let mut targets = VecDeque::new();
        let mut captured = false;

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = event_kind.pointer_id()
        {
            let capture = &app.elements.get_as::<WindowElement>(window).pointer_capture;
            captured = capture.find_pointer_capture_target(event_kind, &pointer_id).is_some();
            let target = find_target(
                root,
                mouse_position,
                event_kind,
                &pointer_id,
                TargetSearchContext {
                    renderer: &*app.elements.get_as::<WindowElement>(window).renderer,
                    target_scratch: &mut app.target_scratch,
                    pointer_capture: capture,
                    elements: &app.elements,
                    by_internal_id: &app.by_internal_id,
                },
            );
            targets = freeze_target_list(target, &app.elements);
        } else if event_kind.is_keyboard_event()
            && let Some(focus) = app.focus
            && app.elements.contains(focus)
        {
            targets = freeze_target_list(focus, &app.elements);
        }

        if targets.is_empty() {
            targets.push_back(root);
        }
        if event_kind.is_system_pointer_event() {
            Self::maybe_dispatch_pointer_leave(&targets, app);
            Self::maybe_dispatch_pointer_enter(&targets, app);
        }

        event_kind.retarget(targets[0]);
        dispatch_event(event_kind, &targets, app);
        let prevented = event_kind.is_default_prevented();
        let dispatched_target =
            matches!(event_kind, EventKind::PointerUp(_) | EventKind::PointerDown(_)).then(|| event_kind.target());

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = event_kind.pointer_id()
        {
            let changed =
                PointerCapture::maybe_handle_implicit_pointer_capture_release(app, window, event_kind, &pointer_id);
            if changed {
                let capture = &app.elements.get_as::<WindowElement>(window).pointer_capture;
                let target = find_target(
                    root,
                    mouse_position,
                    event_kind,
                    &pointer_id,
                    TargetSearchContext {
                        renderer: &*app.elements.get_as::<WindowElement>(window).renderer,
                        target_scratch: &mut app.target_scratch,
                        pointer_capture: capture,
                        elements: &app.elements,
                        by_internal_id: &app.by_internal_id,
                    },
                );
                targets = freeze_target_list(target, &app.elements);
                Self::maybe_dispatch_pointer_leave(&targets, app);
                Self::maybe_dispatch_pointer_enter(&targets, app);
            }
        }

        Self::maybe_dispatch_pointer_click(dispatched_target, captured, event_kind, app);
        if event_kind.is_system_pointer_event() {
            app.event_dispatcher.previous_targets = targets;
        }
        Self::dispatch_queued_events(app);
        prevented
    }
}
