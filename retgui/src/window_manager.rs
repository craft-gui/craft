#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use retgui_renderer::blank_renderer::BlankRenderer;

use rustc_hash::FxHashSet;

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::app::App;
use crate::elements::{AnimationSchedule, DynElement, Elements, Window, WindowElement};
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

    pub(crate) fn any_perf_stats_enabled(&self, elements: &Elements) -> bool {
        self.windows
            .iter()
            .any(|window| elements.get_as::<WindowElement>(window.inner).perf_stats_enabled())
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Elements {
    /// Dirties all gummy nodes and redraws each window.
    pub(crate) fn dirty_and_redraw_all_windows(&mut self, active: bool) {
        if !active {
            return;
        }

        for window_element in &self.window_manager.windows {
            let id = self
                .get(window_element.inner)
                .element_data()
                .layout
                .gummy_node_id
                .unwrap();
            self.gummy_tree.mark_node_and_leaves_dirty(id);
            window_element.request_redraw(self);
        }
    }

    pub(crate) fn on_resume(&mut self, retgui_app: &mut App, event_loop: Option<&dyn ActiveEventLoop>) {
        let now = Instant::now();
        self.apply_pending_animation_updates(now);
        for scheduled in &mut self.window_manager.scheduled_animations {
            scheduled.last_tick = now;
            scheduled.dormant = false;
        }
        for index in 0..self.window_manager.windows.len() {
            let window_element = self.window_manager.windows[index];
            window_element.create(retgui_app, self, event_loop);
        }

        for scheduled in &self.window_manager.scheduled_animations {
            if self.contains(scheduled.element) {
                self.get(scheduled.element).request_window_redraw();
            }
        }
    }

    pub(crate) fn on_about_to_wait(
        &mut self,
        retgui_app: &mut App,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) -> Option<Duration> {
        if !retgui_app.active {
            return None;
        }

        let now = Instant::now();
        self.apply_pending_animation_updates(now);
        let mut next_update = AnimationSchedule::None;

        // Create windows that were created during the program run.
        for index in 0..self.window_manager.windows.len() {
            let window_element = self.window_manager.windows[index];
            if window_element.winit_window(self).is_none() {
                window_element.create(retgui_app, self, event_loop);
            }

            if window_element.redraw_requested(self) {
                window_element.request_redraw(self);
            }

            match self.animation_schedule_for_window(&window_element, now) {
                AnimationSchedule::None => {}
                AnimationSchedule::NextFrame => window_element.request_redraw(self),
                schedule @ AnimationSchedule::At(_) => next_update = next_update.merge(schedule),
            }
        }

        match next_update {
            AnimationSchedule::At(deadline) => Some(deadline.duration_since(now)),
            AnimationSchedule::None | AnimationSchedule::NextFrame => None,
        }
    }

    pub(crate) fn close_window(&mut self, window: &Window) {
        let (manager, elements) = self.disjoint_borrow_window_manager_and_elements();
        manager.scheduled_animations.retain(|scheduled| {
            elements.contains(scheduled.element)
                && elements.get(scheduled.element).element_data().window != Some(window.inner)
        });
        if self.window_manager.windows.iter().any(|w| w.inner == window.inner) {
            self.get_as_mut::<WindowElement>(window.inner).renderer = Box::new(BlankRenderer::default());
            release_window_accessibility(self, window.inner);
            window.set_winit_window(self, None);
            self.window_manager.windows.retain(|w| w.inner != window.inner);
        }
    }

    /// Advances due animations belonging to the window and returns when it next
    /// needs an animation-driven redraw.
    pub(crate) fn animation_tick(&mut self, window: &Window) -> AnimationSchedule {
        self.animation_tick_at(window, Instant::now())
    }

    fn animation_tick_at(&mut self, window: &Window, now: Instant) -> AnimationSchedule {
        self.apply_pending_animation_updates(now);
        let mut retained = 0;
        for index in 0..self.window_manager.scheduled_animations.len() {
            let mut scheduled = self.window_manager.scheduled_animations[index];
            if !self.contains(scheduled.element) {
                continue;
            }
            if self.get(scheduled.element).element_data().window == Some(window.inner) {
                if !animation_is_runnable(self, scheduled.element) {
                    scheduled.dormant = true;
                } else {
                    if scheduled.dormant {
                        scheduled.last_tick = now;
                        scheduled.dormant = false;
                    }

                    let due = match scheduled.deadline {
                        AnimationSchedule::NextFrame => true,
                        AnimationSchedule::At(deadline) => deadline <= now,
                        AnimationSchedule::None => false,
                    };
                    if due {
                        let delta = now.duration_since(scheduled.last_tick);
                        scheduled.last_tick = now;
                        let next = self.dispatch_mut(scheduled.element, |animating, elements| {
                            animating.animation_tick(elements, delta)
                        });
                        if next == AnimationSchedule::None {
                            continue;
                        }
                        scheduled.deadline = next;
                    }
                }
            }
            self.window_manager.scheduled_animations[retained] = scheduled;
            retained += 1;
        }
        self.window_manager.scheduled_animations.truncate(retained);
        self.apply_pending_animation_updates(now);
        self.animation_schedule_for_window(window, now)
    }

    fn apply_pending_animation_updates(&mut self, now: Instant) {
        let (manager, elements) = self.disjoint_borrow_window_manager_and_elements();
        manager
            .scheduled_animations
            .retain(|scheduled| elements.contains(scheduled.element));
        for (element, reset_clock) in self.take_pending_animation_updates() {
            if !self.contains(element) {
                continue;
            }
            let window = self.get(element).element_data().window;
            if window.is_some_and(|window| {
                !self
                    .window_manager
                    .windows
                    .iter()
                    .any(|registered| registered.inner == window)
            }) {
                continue;
            }
            if let Some(scheduled) = self
                .window_manager
                .scheduled_animations
                .iter_mut()
                .find(|scheduled| scheduled.element == element)
            {
                scheduled.deadline = AnimationSchedule::NextFrame;
                if reset_clock {
                    scheduled.last_tick = now;
                }
            } else {
                self.window_manager.scheduled_animations.push(ScheduledAnimation {
                    element,
                    deadline: AnimationSchedule::NextFrame,
                    last_tick: now,
                    dormant: false,
                });
            }
        }
    }

    fn animation_schedule_for_window(&self, window: &Window, now: Instant) -> AnimationSchedule {
        self.window_manager
            .scheduled_animations
            .iter()
            .filter(|scheduled| {
                self.contains(scheduled.element)
                    && self.get(scheduled.element).element_data().window == Some(window.inner)
                    && animation_is_runnable(self, scheduled.element)
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
    let element = elements.get(element);
    let can_change_own_visibility = element.element_data().animations.iter().any(|animation| {
        !animation.is_finished()
            && animation
                .key_frames
                .iter()
                .flat_map(|keyframe| keyframe.styles())
                .any(|style| matches!(style, StyleVariant::Display(_) | StyleVariant::Visible(_)))
    });
    if !element.is_visible() && !can_change_own_visibility {
        return false;
    }

    let mut ancestor = element.element_data().parent;
    while let Some(element) = ancestor {
        let element = elements.get(element);
        if !element.is_visible() {
            return false;
        }
        ancestor = element.element_data().parent;
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
