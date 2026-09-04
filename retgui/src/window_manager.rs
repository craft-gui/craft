#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use retgui_renderer::blank_renderer::BlankRenderer;
use rustc_hash::FxHashSet;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::app::App;
use crate::elements::{AnimationSchedule, DynElement, Elements, Window, WindowNode};
use crate::style::StyleVariant;

#[derive(Clone, Copy, Debug)]
struct ScheduledAnimation {
    element: DynElement,
    deadline: AnimationSchedule,
    last_tick: Instant,
    dormant: bool,
}

pub(crate) struct WindowManager {
    windows: Vec<Window>,
    scheduled_animations: Vec<ScheduledAnimation>,
}

impl WindowManager {
    pub(crate) fn new() -> Self {
        Self {
            windows: Vec::new(),
            scheduled_animations: Vec::new(),
        }
    }

    pub(crate) fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    pub(crate) fn get_window_by_id(&self, elements: &Elements, window_id: WindowId) -> Option<Window> {
        for window in &self.windows {
            if let Some(winit_window) = window.winit_window(elements)
                && winit_window.id() == window_id
            {
                return Some(*window);
            }
        }
        None
    }

    /// Dirties all gummy nodes and redraws each window.
    pub(crate) fn dirty_and_redraw_all_windows(&mut self, elements: &mut Elements, active: bool) {
        if !active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            let id = elements
                .get(window_element.inner)
                .element_data()
                .layout
                .gummy_node_id
                .unwrap();
            elements.gummy_tree.mark_node_and_leaves_dirty(id);
            window_element.request_redraw(elements);
        }
    }

    pub(crate) fn on_resume(
        &mut self,
        retgui_app: &mut App,
        elements: &mut Elements,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) {
        let now = Instant::now();
        self.apply_pending_animation_updates(elements, now);
        for scheduled in &mut self.scheduled_animations {
            scheduled.last_tick = now;
            scheduled.dormant = false;
        }
        for window_element in &self.windows {
            window_element.create(retgui_app, elements, event_loop);
        }

        for scheduled in &self.scheduled_animations {
            if elements.contains(scheduled.element) {
                elements.get(scheduled.element).request_window_redraw();
            }
        }
    }

    pub(crate) fn on_about_to_wait(
        &mut self,
        retgui_app: &mut App,
        elements: &mut Elements,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) -> Option<Duration> {
        if !retgui_app.active {
            return None;
        }

        let now = Instant::now();
        self.apply_pending_animation_updates(elements, now);
        let mut next_update = AnimationSchedule::None;

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            if window_element.winit_window(elements).is_none() {
                window_element.create(retgui_app, elements, event_loop);
            }

            if window_element.redraw_requested(elements) {
                window_element.request_redraw(elements);
            }

            match self.animation_schedule_for_window(elements, window_element, now) {
                AnimationSchedule::None => {}
                AnimationSchedule::NextFrame => window_element.request_redraw(elements),
                schedule @ AnimationSchedule::At(_) => next_update = next_update.merge(schedule),
            }
        }

        match next_update {
            AnimationSchedule::At(deadline) => Some(deadline.duration_since(now)),
            AnimationSchedule::None | AnimationSchedule::NextFrame => None,
        }
    }

    pub(crate) fn any_perf_stats_enabled(&self, elements: &Elements) -> bool {
        self.windows
            .iter()
            .any(|window| elements.get_as::<WindowNode>(window.inner).perf_stats_enabled())
    }

    pub fn close_window(&mut self, elements: &mut Elements, window: &Window) {
        self.scheduled_animations.retain(|scheduled| {
            elements.contains(scheduled.element)
                && elements.get(scheduled.element).element_data().window != Some(window.inner)
        });
        self.windows.retain(|w| {
            let is_target = w.inner == window.inner;

            if is_target {
                elements.get_as_mut::<WindowNode>(w.inner).renderer = Box::new(BlankRenderer::default());
                release_window_accessibility(elements, w.inner);
                w.set_winit_window(elements, None);
            }

            !is_target
        });
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Advances due animations belonging to the window and returns when it next
    /// needs an animation-driven redraw.
    pub(crate) fn animation_tick(&mut self, elements: &mut Elements, window: &Window) -> AnimationSchedule {
        self.animation_tick_at(elements, window, Instant::now())
    }

    fn animation_tick_at(&mut self, elements: &mut Elements, window: &Window, now: Instant) -> AnimationSchedule {
        self.apply_pending_animation_updates(elements, now);
        let mut retained = Vec::with_capacity(self.scheduled_animations.len());
        for mut scheduled in std::mem::take(&mut self.scheduled_animations) {
            if !elements.contains(scheduled.element) {
                continue;
            }
            if elements.get(scheduled.element).element_data().window != Some(window.inner) {
                retained.push(scheduled);
                continue;
            }

            if !animation_is_runnable(elements, scheduled.element) {
                scheduled.dormant = true;
                retained.push(scheduled);
                continue;
            }
            if scheduled.dormant {
                scheduled.last_tick = now;
                scheduled.dormant = false;
            }

            let due = match scheduled.deadline {
                AnimationSchedule::NextFrame => true,
                AnimationSchedule::At(deadline) => deadline <= now,
                AnimationSchedule::None => false,
            };
            if !due {
                retained.push(scheduled);
                continue;
            }

            let delta = now.duration_since(scheduled.last_tick);
            scheduled.last_tick = now;
            let next = elements.dispatch_mut(scheduled.element, |animating, elements| {
                animating.animation_tick(elements, delta)
            });
            if next != AnimationSchedule::None {
                scheduled.deadline = next;
                retained.push(scheduled);
            }
        }
        self.scheduled_animations = retained;

        // Ticks may schedule or stop animations, including on other elements.
        // Apply those requests after restoring the active registry so none are
        // lost through nested element dispatch.
        self.apply_pending_animation_updates(elements, now);
        self.animation_schedule_for_window(elements, window, now)
    }

    fn apply_pending_animation_updates(&mut self, elements: &mut Elements, now: Instant) {
        self.scheduled_animations
            .retain(|scheduled| elements.contains(scheduled.element));
        for (element, reset_clock) in elements.take_pending_animation_updates() {
            if !elements.contains(element) {
                continue;
            }
            let window = elements.get(element).element_data().window;
            if window.is_some_and(|window| !self.windows.iter().any(|registered| registered.inner == window)) {
                continue;
            }
            if let Some(scheduled) = self
                .scheduled_animations
                .iter_mut()
                .find(|scheduled| scheduled.element == element)
            {
                scheduled.deadline = AnimationSchedule::NextFrame;
                if reset_clock {
                    scheduled.last_tick = now;
                }
            } else {
                self.scheduled_animations.push(ScheduledAnimation {
                    element,
                    deadline: AnimationSchedule::NextFrame,
                    last_tick: now,
                    dormant: false,
                });
            }
        }
    }

    fn animation_schedule_for_window(&self, elements: &Elements, window: &Window, now: Instant) -> AnimationSchedule {
        self.scheduled_animations
            .iter()
            .filter(|scheduled| {
                elements.contains(scheduled.element)
                    && elements.get(scheduled.element).element_data().window == Some(window.inner)
                    && animation_is_runnable(elements, scheduled.element)
            })
            .fold(AnimationSchedule::None, |schedule, scheduled| {
                let next = match scheduled.deadline {
                    AnimationSchedule::NextFrame => AnimationSchedule::NextFrame,
                    AnimationSchedule::At(deadline) if deadline <= now => AnimationSchedule::NextFrame,
                    AnimationSchedule::At(deadline) => AnimationSchedule::At(deadline),
                    AnimationSchedule::None => AnimationSchedule::None,
                };
                schedule.merge(next)
            })
    }
}

fn animation_is_runnable(elements: &Elements, element: DynElement) -> bool {
    let node = elements.get(element);
    let can_change_own_visibility = node.element_data().animations.iter().any(|animation| {
        !animation.is_finished()
            && animation
                .key_frames
                .iter()
                .flat_map(|keyframe| keyframe.styles())
                .any(|style| matches!(style, StyleVariant::Display(_) | StyleVariant::Visible(_)))
    });
    if !node.is_visible() && !can_change_own_visibility {
        return false;
    }

    let mut ancestor = node.element_data().parent;
    while let Some(element) = ancestor {
        let node = elements.get(element);
        if !node.is_visible() {
            return false;
        }
        ancestor = node.element_data().parent;
    }
    true
}

fn release_window_accessibility(elements: &mut Elements, window: DynElement) {
    let (access_tree, root) = {
        let data = elements.get(window).element_data();
        (data.access_tree.clone(), data.access_root)
    };

    if let Some(root) = root {
        access_tree.remove_node(root);
    }

    let mut pending = vec![window];
    let mut visited = FxHashSet::default();
    while let Some(element) = pending.pop() {
        if !visited.insert(element) || !elements.contains(element) {
            continue;
        }
        let data = elements.get_mut(element).element_data_mut();
        pending.extend(data.children.iter().copied());
        data.access_key = None;
        data.access_root = None;
    }
}
