use retgui_primitives::geometry::Point;
use retgui_renderer::renderer::Renderer;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

use crate::app::{FOCUS, dequeue_event};
use crate::elements::ElementInternals;
use crate::events::helpers::{call_user_event_handlers, find_target, freeze_target_list};
use crate::events::{Event, EventKind};
use crate::text::text_context::TextContext;


pub (super) fn dispatch_event(event: &mut Event, event_kind: &EventKind, targets: &VecDeque<Rc<RefCell<dyn ElementInternals>>>, text_context: &mut TextContext) {
    // Bubbling
    for current_target in targets.iter() {
        event.current_target = current_target.clone();
        call_user_event_handlers(event, event_kind);
        if !event.propagate {
            break;
        }
    }

    if !event.prevent_defaults {
        // Call the default on_event element functions.
        for current_target in targets.iter() {
            event.current_target = current_target.clone();
            current_target.borrow_mut().on_event(event_kind, text_context, event);
            if !event.propagate {
                break;
            }
        }
    }
}

pub (super) fn dispatch_event_once(event: &mut Event, event_kind: &EventKind, text_context: &mut TextContext) {
    call_user_event_handlers(event, event_kind);

    if !event.prevent_defaults {
        let target = event.target.clone();
        target.borrow_mut().on_event(event_kind, text_context, event);
    }
}

/// Responsible for dispatching events.
pub(crate) struct EventDispatcher {
    /// A "frozen" target list used to diff against the current target list.
    /// This is useful for pointer enter, leave, etc.
    previous_targets: VecDeque<Weak<RefCell<dyn ElementInternals>>>,
}

impl EventDispatcher {
    /// Creates an event dispatcher and zeros out the previous target list.
    pub fn new() -> Self {
        Self {
            previous_targets: Default::default(),
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
                let mut event = Event::new(prev_target.clone());
                let event_kind = EventKind::PointerLeave();
                dispatch_event_once(&mut event, &event_kind, text_context);
            }
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
                let mut event = Event::new(target.clone());
                let event_kind = EventKind::PointerEnter();
                dispatch_event_once(&mut event, &event_kind, text_context);
            }
        }
    }

    /// Dispatches events.
    /// May emit multiple events from a single message (pointer enter, leave, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_event(
        &mut self,
        event_kind: &EventKind,
        mouse_position: Option<Point>,
        root: Rc<RefCell<dyn ElementInternals>>,
        text_context: &mut TextContext,
        renderer: &mut dyn Renderer,
        target_scratch: &mut Vec<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        let pointer_capture = root
            .borrow()
            .element_data()
            .window
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|w| w.borrow().pointer_capture.clone())
            .unwrap();

        let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();

        if event_kind.is_system_pointer_event()
            && let Some(pointer_id) = &event_kind.pointer_id()
        {
            // Find the target and freeze the list, so the same set of elements are visited across sub event dispatches.
            let target: Rc<RefCell<dyn ElementInternals>> = find_target(
                &root,
                mouse_position,
                event_kind,
                renderer,
                target_scratch,
                &pointer_capture.borrow(),
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

        let mut system_event = Event::new(targets[0].clone());
        dispatch_event(&mut system_event, event_kind, &mut targets, text_context);

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

        if event_kind.is_system_pointer_event() {
            self.previous_targets = targets.iter().map(Rc::downgrade).collect();
        }

        // Handle Semantic Events (DropdownItemSelected, Click, and etc.)
        while let Some((mut event, message)) = dequeue_event() {
            let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = freeze_target_list(event.target.clone());
            dispatch_event(&mut event, &message, &mut targets, text_context);
        }
    }
}
