/*use crate::engine::app_message::AppMessage;
use crate::engine::events::internal::InternalMessage;
use futures::channel::mpsc::channel;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::{channel, SinkExt, StreamExt};
use log::info;
use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::web::WindowAttributesExtWeb;
use winit::window::{WindowAttributes, WindowId};

#[cfg(target_arch = "wasm32")]
pub fn wasm_main() {
    let (mut tx, mut rx) = channel::<AppMessage>(100);

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    let mut x: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    let mut y = x.clone();

    wasm_bindgen_futures::spawn_local(async move {
        *y.borrow_mut() += 1;
        //tx.send(AppMessage::new(0, InternalMessage::Confirmation)).await.unwrap();
        //tx.send(AppMessage::new(0, InternalMessage::Confirmation)).await.unwrap();
    });
    x.borrow_mut().add_assign(1);

    wasm_bindgen_futures::spawn_local(async move {
        info!("starting main loop.");
        while let Some(message) = rx.next().await {
            info!("message gotten");
        }
    });

    event_loop
        .run_app(WasmApp {
            app_sender: tx.clone(),
        })
        .expect("Failed to run event_loop.");
}

struct WasmApp {
    app_sender: Sender<AppMessage>,
}

impl ApplicationHandler for WasmApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            WindowAttributes::default().with_title("wasm").with_canvas(Some(canvas))
        };

        event_loop.create_window(window_attributes).expect("Failed to create window.");
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        info!("event: {:?}", event);
        let mut tx = self.app_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            tx.send(AppMessage::new(0, InternalMessage::Confirmation)).await;
        });
    }
}
*/
