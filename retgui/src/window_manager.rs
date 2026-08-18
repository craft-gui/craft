use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{App, GUMMY_TREE};
use crate::elements::{ElementData, Window};
use retgui_renderer::blank_renderer::BlankRenderer;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub(crate) struct WindowManager {
    windows: Vec<Window>,
}

impl WindowManager {
    pub(crate) fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    pub(crate) fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    pub(crate) fn get_window_by_id(&self, window_id: WindowId) -> Option<Window> {
        for window in &self.windows {
            if let Some(winit_window) = window.winit_window()
                && winit_window.id() == window_id
            {
                return Some(window.clone());
            }
        }
        None
    }

    /// Dirties all gummy nodes and redraws each window.
    pub(crate) fn dirty_and_redraw_all_windows(&mut self, retgui_app: &mut App) {
        if !retgui_app.active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            let id = window_element
                .inner
                .borrow_mut()
                .element_data()
                .layout
                .gummy_node_id
                .unwrap();
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.mark_node_and_leaves_dirty(id);
            });
            window_element.request_redraw();
        }
    }

    pub(crate) fn on_resume(&mut self, retgui_app: &mut App, event_loop: Option<&ActiveEventLoop>) {
        for window_element in &self.windows {
            window_element.create(retgui_app, event_loop);
        }
    }

    pub(crate) fn on_about_to_wait(&mut self, retgui_app: &mut App, event_loop: Option<&ActiveEventLoop>) {
        if !retgui_app.active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            if window_element.winit_window().is_none() {
                window_element.create(retgui_app, event_loop);
            }
        }
    }

    pub(crate) fn any_perf_stats_enabled(&self) -> bool {
        self.windows
            .iter()
            .any(|window| window.inner.borrow().perf_stats_enabled())
    }

    pub fn close_window(&mut self, window: &Window) {
        self.windows.retain(|w| {
            let is_target = Rc::ptr_eq(&w.inner, &window.inner);

            if is_target {
                w.set_winit_window(None);
                w.inner.borrow_mut().renderer = Rc::new(RefCell::new(BlankRenderer::default()));
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

    pub(crate) fn clear(&mut self) {
        self.windows.clear();
    }
}
