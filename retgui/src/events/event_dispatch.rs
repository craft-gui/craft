use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};

use crate::events::{PointerButton, PointerId};

use retgui_primitives::geometry::Point;

use retgui_renderer::renderer::Renderer;

use crate::app::{FOCUS, dequeue_event};
use crate::elements::{DynElement, ElementInternals};
use crate::events::helpers::{call_user_event_handlers, find_target, freeze_target_list, nearest_common_ancestor};
use crate::events::{ClickEvent, ClickTrigger, Event, EventKind, PointerEnterEvent, PointerLeaveEvent};
use crate::text::text_context::TextContext;

pub(super) fn dispatch_event(
    event: &mut EventKind,
    targets: &VecDeque<Rc<RefCell<dyn ElementInternals>>>,
    text_context: &mut TextContext,
) {
    // Capture Phase
    for current_target in targets.iter().rev() {
        event.base_mut().current_target = DynElement::new(current_target.clone());
        call_user_event_handlers(event, true);
        if event.is_propagation_stopped() {
            break;
        }
    }

    // Bubble phase
    if !event.is_propagation_stopped() {
        for current_target in targets.iter() {
            event.base_mut().current_target = DynElement::new(current_target.clone());
            call_user_event_handlers(event, false);
            if event.is_propagation_stopped() {
                break;
            }
        }
    }

    if !event.is_default_prevented() {
        // Call the default on_event element functions.
        for current_target in targets.iter() {
            event.base_mut().current_target = DynElement::new(current_target.clone());
            current_target.borrow_mut().on_event(event, text_context);
            if event.is_propagation_stopped() {
                break;
            }
        }
    }
}

pub(super) fn dispatch_event_once(event: &mut EventKind, text_context: &mut TextContext) {
    call_user_event_handlers(event, true);
    if !event.is_propagation_stopped() {
        call_user_event_handlers(event, false);
    }

    if !event.is_default_prevented() {
        let target = event.target();
        target.inner.borrow_mut().on_event(event, text_context);
    }
}

/// Responsible for dispatching events.
pub(crate) struct EventDispatcher {
    /// A "frozen" target list used to diff against the current target list.
    /// This is useful for pointer enter, leave, etc.
    previous_targets: VecDeque<Weak<RefCell<dyn ElementInternals>>>,
    active_pointer_targets: HashMap<PointerId, Weak<RefCell<dyn ElementInternals>>>,
}

impl EventDispatcher {
    /// Creates an event dispatcher and zeros out the previous target list.
    pub fn new() -> Self {
        Self {
            previous_targets: Default::default(),
            active_pointer_targets: Default::default(),
        }
    }

    /// Dispatch all queued events.
    pub(crate) fn dispatch_queued_events(&mut self, text_context: &mut TextContext) {
        while let Some(mut event) = dequeue_event() {
            let targets = freeze_target_list(event.target().inner);
            dispatch_event(&mut event, &targets, text_context);
        }
    }

    /// Diffs the current and previous target lists and dispatches
    /// `pointer_leave` to any element that was present in the previous list
    /// but is not present in the current one.
    ///
    /// Note: This event does not bubble.
    pub(super) fn maybe_dispatch_pointer_leave(
        &self,
        text_context: &mut TextContext,
        targets: &VecDeque<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        for prev_target in self.previous_targets.iter() {
            let mut found = false;

            let prev_target = prev_target.upgrade();
            if prev_target.is_none() {
                continue;
            }
            let prev_target = prev_target.unwrap();

            let prev_target_id = prev_target.borrow().id();

            for target in targets.iter() {
                let target_id = target.borrow().id();

                if prev_target_id == target_id {
                    found = true;
                    break;
                }
            }

            // We had a prev target, but we don't in the new list. (PointerLeave)
            if !found {
                let mut event = EventKind::PointerLeave(PointerLeaveEvent::new(DynElement::new(prev_target.clone())));
                dispatch_event_once(&mut event, text_context);
            }
        }
    }

    fn maybe_dispatch_pointer_click(
        &mut self,
        dispatched_pointer_up_down_target: Option<Rc<RefCell<dyn ElementInternals>>>,
        target_was_pointer_captured: bool,
        event_kind: &EventKind,
        text_context: &mut TextContext,
    ) {
        match event_kind {
            EventKind::PointerDown(pb) if pb.pointer.is_primary_pointer() && pb.button == Some(PointerButton::Left) => {
                if let Some(pointer_id) = pb.pointer.pointer_id {
                    let down_target = dispatched_pointer_up_down_target.unwrap();
                    self.active_pointer_targets
                        .insert(pointer_id, Rc::downgrade(&down_target));
                }
            }

            EventKind::PointerUp(pb) if pb.pointer.is_primary_pointer() && pb.button == Some(PointerButton::Left) => {
                let pointer_id = pb.pointer.pointer_id.unwrap();
                if let Some(down_target) = self
                    .active_pointer_targets
                    .get(&pointer_id)
                    .and_then(|target| target.upgrade())
                {
                    let up_target = dispatched_pointer_up_down_target.unwrap();

                    let click_target = if target_was_pointer_captured {
                        Some(up_target.clone())
                    } else {
                        nearest_common_ancestor(&down_target, &up_target)
                    };

                    if let Some(click_target) = click_target {
                        let trigger = ClickTrigger::Pointer {
                            button: pb.button,
                            position: pb.position,
                        };
                        let mut click_event =
                            EventKind::Click(ClickEvent::new(DynElement::new(click_target.clone()), trigger));
                        let click_targets = freeze_target_list(click_target);

                        dispatch_event(&mut click_event, &click_targets, text_context);
                    }
                }

                self.active_pointer_targets.remove(&pointer_id);
            }

            _ => {}
        }
    }

    /// Diffs the current and previous target lists and dispatches
    /// `pointer_enter` to any element that exists in the current list
    /// but not in the previous one.
    ///
    /// Note: This event does not bubble.
    pub(super) fn maybe_dispatch_pointer_enter(
        &self,
        text_context: &mut TextContext,
        targets: &VecDeque<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        for target in targets.iter().rev() {
            let mut found = false;
            let target_id = target.borrow().id();

            for prev_target in self.previous_targets.iter().rev() {
                let prev_target = prev_target.upgrade();
                if prev_target.is_none() {
                    continue;
                }
                let prev_target = prev_target.unwrap();
                let prev_target_id = prev_target.borrow().id();

                if prev_target_id == target_id {
                    found = true;
                    break;
                }
            }

            // We weren't in the prev target list, but we are in the new list. (PointerEnter)
            if !found {
                let mut event = EventKind::PointerEnter(PointerEnterEvent::new(DynElement::new(target.clone())));
                dispatch_event_once(&mut event, text_context);
            }
        }
    }

    /// Dispatches events.
    /// May emit multiple events from a single message (pointer enter, leave, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_event(
        &mut self,
        event_kind: &mut EventKind,
        mouse_position: Option<Point>,
        root: Rc<RefCell<dyn ElementInternals>>,
        text_context: &mut TextContext,
        renderer: &mut dyn Renderer,
        target_scratch: &mut Vec<Rc<RefCell<dyn ElementInternals>>>,
    ) -> bool {
        let pointer_capture = root
            .borrow()
            .element_data()
            .window
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|w| w.borrow().pointer_capture.clone())
            .unwrap();

        let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();
        let mut is_pointer_captured = false;

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = &event_kind.pointer_id()
        {
            let pointer_capture = pointer_capture.borrow();
            is_pointer_captured = pointer_capture
                .find_pointer_capture_target(event_kind, pointer_id)
                .is_some();
            // Find the target and freeze the list, so the same set of elements are visited across sub event dispatches.
            let target: Rc<RefCell<dyn ElementInternals>> = find_target(
                &root,
                mouse_position,
                event_kind,
                renderer,
                target_scratch,
                &pointer_capture,
                pointer_id,
            );
            targets = freeze_target_list(target);
        } else if event_kind.is_keyboard_event() {
            FOCUS.with(|f| {
                let focus_ref = f.borrow();
                if let Some(focus_ref) = focus_ref.clone()
                    && let Some(focus) = focus_ref.upgrade()
                {
                    targets = freeze_target_list(focus);
                }
            });
        }

        if targets.is_empty() {
            targets.push_back(root.clone());
        }

        if event_kind.is_system_pointer_event() {
            self.maybe_dispatch_pointer_leave(text_context, &targets);
            self.maybe_dispatch_pointer_enter(text_context, &targets);
        }

        event_kind.retarget(DynElement::new(targets[0].clone()));
        dispatch_event(event_kind, &targets, text_context);
        let prevent_defaults = event_kind.is_default_prevented();

        let dispatched_pointer_up_down_target =
            if matches!(event_kind, EventKind::PointerUp(_) | EventKind::PointerDown(_)) {
                Some(event_kind.target().inner)
            } else {
                None
            };

        // NOTE: May dispatch gotpointercapture and lostpointercapture. Handles capturing and bubbling.
        // The event dispatch flow looks like this:
        // - pointer_event(capture), pointer_event(bubble) (Executed above)
        // - lostpointercapture(capture), lostpointercapture(bubble)
        // - gotpointercapture(capture), gotpointercapture(bubble)
        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = event_kind.pointer_id()
        {
            let did_pointer_capture_change = pointer_capture
                .borrow_mut()
                .maybe_handle_implicit_pointer_capture_release(event_kind, text_context, &pointer_id);

            if did_pointer_capture_change {
                let target: Rc<RefCell<dyn ElementInternals>> = find_target(
                    &root,
                    mouse_position,
                    event_kind,
                    renderer,
                    target_scratch,
                    &pointer_capture.borrow(),
                    &pointer_id,
                );
                targets = freeze_target_list(target);
                self.maybe_dispatch_pointer_leave(text_context, &targets);
                self.maybe_dispatch_pointer_enter(text_context, &targets);
            }
        }

        self.maybe_dispatch_pointer_click(
            dispatched_pointer_up_down_target,
            is_pointer_captured,
            event_kind,
            text_context,
        );

        if event_kind.is_system_pointer_event() {
            self.previous_targets = targets.iter().map(Rc::downgrade).collect();
        }

        // Handle Semantic Events (DropdownItemSelected, Click, and etc.)
        self.dispatch_queued_events(text_context);

        prevent_defaults
    }
}
