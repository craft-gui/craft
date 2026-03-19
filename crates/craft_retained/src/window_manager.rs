use std::rc::Rc;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::app::App;
use crate::elements::Window;

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
            let winit_window = window.winit_window();
            if winit_window.is_some() && winit_window.unwrap().id() == window_id {
                return Some(window.clone());
            }
        }

        None
    }

    // Improve this.
    pub(crate) fn redraw_all(&mut self, craft_app: &mut App) {
        if !craft_app.active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            if let Some(winit_window) = window_element.winit_window() {
                winit_window.request_redraw();
            }
        }
    }

    pub(crate) fn on_resume(&mut self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        for window_element in &self.windows {
            window_element.create(craft_app, event_loop);
        }
    }

    pub(crate) fn on_about_to_wait(&mut self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        if !craft_app.active {
            return;
        }

        // Create windows that were created during the program run.
        for window_element in &self.windows {
            if window_element.winit_window().is_none() {
                window_element.create(craft_app, event_loop);
            }
        }
    }

    pub fn close_window(&mut self, window: &Window) {
        self.windows.retain(|w| {
            let is_target = Rc::ptr_eq(&w.inner, &window.inner);

            if is_target {
                w.set_winit_window(None);
                w.inner.borrow_mut().renderer = None;
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
}
