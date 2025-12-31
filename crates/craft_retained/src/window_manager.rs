use std::cell::RefCell;
use std::rc::Rc;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use crate::elements::Window;

pub(crate) struct WindowManager {
    windows: Vec<Rc<RefCell<Window>>>,
}

impl WindowManager {

    pub(crate) fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    pub(crate) fn create_windows(&mut self, event_loop: &ActiveEventLoop) {
        for window_element in &self.windows {
            println!("Creating window");
            let winit_window = event_loop.create_window(WindowAttributes::default()).expect("Failed to create window");
            winit_window.set_visible(true);
        }
    }

    pub(crate) fn add_window(&mut self, window: Rc<RefCell<Window>>) {
        self.windows.push(window);
    }

}