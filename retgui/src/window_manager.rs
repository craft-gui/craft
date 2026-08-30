use std::time::Duration;

use retgui_renderer::blank_renderer::BlankRenderer;
use rustc_hash::FxHashSet;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::app::App;
use crate::elements::{DynElement, Elements, Window, WindowNode};

pub(crate) struct WindowManager {
    windows: Vec<Window>,
    animating_elements: Vec<DynElement>,
}

impl WindowManager {
    pub(crate) fn new() -> Self {
        Self {
            windows: Vec::new(),
            animating_elements: Vec::new(),
        }
    }

    pub(crate) fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    /// Schedules an element for animation updates. If the window
    pub fn schedule_element_animations(&mut self, element: DynElement) {
        if !self.animating_elements.contains(&element) {
            self.animating_elements.push(element);
        }
    }

    /// Cancel an element for animation updates.
    pub fn cancel_element_animations(&mut self, element: &DynElement) {
        self.animating_elements
            .retain(|registered_element| registered_element != element);
    }

    pub(crate) fn get_window_by_id(&self, elements: &Elements, window_id: WindowId) -> Option<Window> {
        for window in &self.windows {
            if let Some(winit_window) = window.winit_window(elements)
                && winit_window.id() == window_id
            {
                return Some(window.clone());
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
        for window_element in &self.windows {
            window_element.create(retgui_app, elements, event_loop);
            window_element.reset_animation_clock(elements);
        }

        for animating in &self.animating_elements {
            if elements.contains(*animating) {
                elements.get(*animating).request_window_redraw();
            }
        }
    }

    pub(crate) fn on_about_to_wait(
        &mut self,
        retgui_app: &mut App,
        elements: &mut Elements,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) {
        if !retgui_app.active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            if window_element.winit_window(elements).is_none() {
                window_element.create(retgui_app, elements, event_loop);
            }

            // Descendant mutations only have the shared redraw signal; they
            // cannot borrow the window from the element store. Turn that
            // retained invalidation into an actual native redraw request
            // before the event loop goes back to sleep.
            if window_element.redraw_requested(elements) {
                window_element.request_redraw(elements);
            }
        }
    }

    pub(crate) fn any_perf_stats_enabled(&self, elements: &Elements) -> bool {
        self.windows
            .iter()
            .any(|window| elements.get_as::<WindowNode>(window.inner).perf_stats_enabled())
    }

    pub fn close_window(&mut self, elements: &mut Elements, window: &Window) {
        self.windows.retain(|w| {
            let is_target = w.inner == window.inner;

            if is_target {
                // The renderer and accessibility tree both retain native
                // window resources. Release all three owners; clearing only
                // WindowNode::winit_window leaves secondary windows alive.
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

    /// Advances animations belonging to the window being redrawn and returns
    /// whether that window still has active animations.
    pub(crate) fn animation_tick(&mut self, elements: &mut Elements, window: &Window, delta: Duration) -> bool {
        let mut has_active_animations = false;
        let mut retained = Vec::with_capacity(self.animating_elements.len());
        for animating in std::mem::take(&mut self.animating_elements) {
            if !elements.contains(animating) {
                continue;
            }
            elements.dispatch_mut(animating, |animating, _| {
                if animating.element_data().window == Some(window.inner) {
                    animating.animation_tick(delta);
                    has_active_animations |= animating
                        .element_data()
                        .animations
                        .iter()
                        .any(|animation| !animation.is_finished());
                }
            });
            retained.push(animating);
        }
        self.animating_elements = retained;
        has_active_animations
    }
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
