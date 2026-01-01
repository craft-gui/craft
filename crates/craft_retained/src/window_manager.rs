use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};
use craft_renderer::renderer::Renderer;
use crate::app::WINDOW_MANAGER;
use crate::craft_winit_state::CraftState;
use crate::elements::{Element, Window};
use crate::elements::core::ElementData;

pub(crate) struct WindowManager {
    windows: Vec<Rc<RefCell<Window>>>,
}

impl WindowManager {

    pub fn get_main_window() -> Rc<RefCell<Window>> {
        WINDOW_MANAGER.with_borrow(|window_manager| {
           window_manager.windows[0].clone()
        })
    }

    pub(crate) fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    pub(crate) fn add_window(&mut self, window: Rc<RefCell<Window>>) {
        self.windows.push(window);
    }

    pub(crate) fn get_window_by_id(&self, window_id: WindowId) -> Option<Rc<RefCell<Window>>> {
        for window in &self.windows {
            let winit_window = window.borrow_mut().winit_window();
            if winit_window.is_some() && winit_window.unwrap().id() == window_id {
                return Some(window.clone());
            }
        }

        None
    }

    pub(crate) fn on_resume(&mut self, craft_state: &mut CraftState, event_loop: &ActiveEventLoop) {
        for window_element in &self.windows {

            println!("Creating window");
            let winit_window = Arc::new(event_loop.create_window(WindowAttributes::default()).expect("Failed to create window"));
            winit_window.set_visible(true);
            window_element.borrow_mut().set_winit_window(winit_window.clone());

            let renderer_type = craft_state.craft_options.renderer;

            cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    let renderer = craft_state.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Box<dyn Renderer> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                window_element.borrow_mut().renderer = Some(renderer);
            } else {
                let app_sender = craft_state.app_sender.clone();
                let window_copy_2 = window_copy.clone();
                craft_state.runtime.spawn(async move {
                    let renderer: Box<dyn Renderer> = renderer_type.create(window_copy).await;
                    app_sender
                        .send(InternalMessage::RendererCreated(window_copy_2, renderer))
                        .await
                        .expect("Failed to send RendererCreated message");
                });
            }
        }
        }
    }

    pub fn close_window(&mut self, window: &Rc<RefCell<Window>>) {
        self.windows.retain(|w| {
            let is_target = Rc::ptr_eq(w, window);

            if is_target {
                w.borrow_mut().winit_window = None;
                w.borrow_mut().renderer = None;
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