use std::collections::VecDeque;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::Sender;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use retgui_renderer::blank_renderer::BlankRenderer;

use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceId, ResourceManager};

use retgui_runtime::RetGuiRuntime;

use rustc_hash::FxHashSet;

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[cfg(target_arch = "wasm32")]
use crate::app::CreatedRenderer;
use crate::elements::{AnimationSchedule, DynElement, ElementStates, RetainedElements, Window, WindowElement};
use crate::layout::GummyTree;
use crate::style::StyleVariant;
use crate::text::text_context::TextContext;

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

    pub(crate) fn get_window_by_id(&self, elements: &RetainedElements, window_id: WindowId) -> Option<Window> {
        for window in &self.windows {
            if let Some(winit_window) = elements.get_as::<WindowElement>(window.inner).winit_window()
                && winit_window.id() == window_id
            {
                return Some(*window);
            }
        }
        None
    }

    pub(crate) fn any_perf_stats_enabled(&self, elements: &RetainedElements) -> bool {
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

    /// Dirties all gummy nodes and redraws each window.
    pub(crate) fn dirty_and_redraw_all_windows(
        &self,
        elements: &RetainedElements,
        gummy_tree: &mut GummyTree,
        active: bool,
    ) {
        if !active {
            return;
        }

        for window_element in &self.windows {
            let id = elements
                .get(window_element.inner)
                .element_data()
                .layout
                .gummy_node_id
                .unwrap();
            gummy_tree.mark_node_and_leaves_dirty(id);
            elements.get_as::<WindowElement>(window_element.inner).request_redraw();
        }
    }

    pub(crate) fn on_resume(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &ElementStates,
        text_context: &mut TextContext,
        resource_manager: &Arc<ResourceManager>,
        runtime: &mut RetGuiRuntime,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
        #[cfg(target_arch = "wasm32")] created_renderer_sender: &Sender<CreatedRenderer>,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) {
        let now = Instant::now();
        self.apply_pending_animation_updates(elements, pending_animation_updates, now);
        for scheduled in &mut self.scheduled_animations {
            scheduled.last_tick = now;
            scheduled.dormant = false;
        }
        for index in 0..self.windows.len() {
            let window_element = self.windows[index];
            WindowElement::create_window(
                elements,
                gummy_tree,
                states,
                text_context,
                resource_manager.clone(),
                runtime,
                #[cfg(target_arch = "wasm32")]
                created_renderer_sender,
                event_loop,
                window_element.inner,
            );
        }

        for scheduled in &self.scheduled_animations {
            if elements.contains(scheduled.element) {
                elements.get(scheduled.element).request_window_redraw();
            }
        }
    }

    pub(crate) fn on_about_to_wait(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &ElementStates,
        text_context: &mut TextContext,
        resource_manager: &Arc<ResourceManager>,
        runtime: &mut RetGuiRuntime,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
        #[cfg(target_arch = "wasm32")] created_renderer_sender: &Sender<CreatedRenderer>,
        active: bool,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) -> Option<Duration> {
        if !active {
            return None;
        }

        let now = Instant::now();
        self.apply_pending_animation_updates(elements, pending_animation_updates, now);
        let mut next_update = AnimationSchedule::None;

        // Create windows that were created during the program run.
        for index in 0..self.windows.len() {
            let window_element = self.windows[index];
            if elements
                .get_as::<WindowElement>(window_element.inner)
                .winit_window()
                .is_none()
            {
                WindowElement::create_window(
                    elements,
                    gummy_tree,
                    states,
                    text_context,
                    resource_manager.clone(),
                    runtime,
                    #[cfg(target_arch = "wasm32")]
                    created_renderer_sender,
                    event_loop,
                    window_element.inner,
                );
            }

            if elements
                .get_as::<WindowElement>(window_element.inner)
                .redraw_requested()
            {
                elements.get_as::<WindowElement>(window_element.inner).request_redraw();
            }

            match self.animation_schedule_for_window(elements, &window_element, now) {
                AnimationSchedule::None => {}
                AnimationSchedule::NextFrame => elements.get_as::<WindowElement>(window_element.inner).request_redraw(),
                schedule @ AnimationSchedule::At(_) => next_update = next_update.merge(schedule),
            }
        }

        match next_update {
            AnimationSchedule::At(deadline) => Some(deadline.duration_since(now)),
            AnimationSchedule::None | AnimationSchedule::NextFrame => None,
        }
    }

    pub(crate) fn close_window(&mut self, elements: &mut RetainedElements, window: &Window) {
        self.scheduled_animations.retain(|scheduled| {
            elements.contains(scheduled.element)
                && elements.get(scheduled.element).element_data().window != Some(window.inner)
        });
        if self.windows.iter().any(|w| w.inner == window.inner) {
            elements.get_as_mut::<WindowElement>(window.inner).renderer = Box::new(BlankRenderer::default());
            release_window_accessibility(elements, window.inner);
            elements
                .get_as_mut::<WindowElement>(window.inner)
                .set_winit_window(None);
            self.windows.retain(|w| w.inner != window.inner);
        }
    }

    /// Advances due animations belonging to the window and returns when it next
    /// needs an animation-driven redraw.
    pub(crate) fn animation_tick(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
        window: &Window,
    ) -> AnimationSchedule {
        self.animation_tick_at(
            elements,
            gummy_tree,
            states,
            pending_resources,
            pending_animation_updates,
            window,
            Instant::now(),
        )
    }

    fn animation_tick_at(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
        window: &Window,
        now: Instant,
    ) -> AnimationSchedule {
        self.apply_pending_animation_updates(elements, pending_animation_updates, now);
        let mut retained = 0;
        for index in 0..self.scheduled_animations.len() {
            let mut scheduled = self.scheduled_animations[index];
            if !elements.contains(scheduled.element) {
                continue;
            }
            if elements.get(scheduled.element).element_data().window == Some(window.inner) {
                if !animation_is_runnable(elements, scheduled.element) {
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
                        let next = elements.dispatch_mut(scheduled.element, |animating, elements| {
                            animating.animation_tick(elements, gummy_tree, states, pending_resources, delta)
                        });
                        if next == AnimationSchedule::None {
                            continue;
                        }
                        scheduled.deadline = next;
                    }
                }
            }
            self.scheduled_animations[retained] = scheduled;
            retained += 1;
        }
        self.scheduled_animations.truncate(retained);
        self.apply_pending_animation_updates(elements, pending_animation_updates, now);
        self.animation_schedule_for_window(elements, window, now)
    }

    fn apply_pending_animation_updates(
        &mut self,
        elements: &RetainedElements,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
        now: Instant,
    ) {
        self.scheduled_animations
            .retain(|scheduled| elements.contains(scheduled.element));
        for (element, reset_clock) in pending_animation_updates.drain(..) {
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

    fn animation_schedule_for_window(
        &self,
        elements: &RetainedElements,
        window: &Window,
        now: Instant,
    ) -> AnimationSchedule {
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

fn animation_is_runnable(elements: &RetainedElements, element: DynElement) -> bool {
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

fn release_window_accessibility(elements: &mut RetainedElements, window: DynElement) {
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
