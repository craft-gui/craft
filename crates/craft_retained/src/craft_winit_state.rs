#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    winit::platform::web::WindowAttributesExtWebSys,
};

#[cfg(target_arch = "wasm32")]
use {crate::wasm_queue::WasmQueue, crate::wasm_queue::WASM_QUEUE};

use crate::events::internal::InternalMessage;
use craft_renderer::renderer::Renderer;
use crate::{CraftOptions};
use craft_logging::info;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use web_time as time;

use craft_runtime::Receiver;
use craft_runtime::Sender;
use craft_runtime::CraftRuntimeHandle;

use crate::app::{App, CURRENT_WINDOW_ID, DOCUMENTS};
use std::sync::Arc;
use ui_events::pointer::{PointerEvent};
use ui_events_winit::{WindowEventReducer, WindowEventTranslation};
use winit::dpi::LogicalSize;
use crate::document::Document;

/// Stores state related to Winit.
///
/// Forwards most events to the main Craft Event Loop.
pub struct CraftState {
    #[allow(dead_code)]
    pub runtime: CraftRuntimeHandle,
    pub wait_cancelled: bool,
    pub close_requested: bool,
    #[allow(dead_code)]
    pub winit_receiver: Receiver<InternalMessage>,
    #[allow(dead_code)]
    pub app_sender: Sender<InternalMessage>,
    pub craft_options: CraftOptions,
    pub event_reducer: WindowEventReducer,
    pub craft_app: Box<App>,
}

pub(crate) struct CraftWinitState {
    craft_state: CraftState,
}

impl CraftWinitState {
    pub(crate) fn new(craft_state: CraftState) -> Self {
        Self {
            craft_state,
        }
    }
}

impl ApplicationHandler for CraftWinitState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.craft_state.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let craft_state = &mut self.craft_state;
        
        let mut window_attributes =
            WindowAttributes::default().with_title(craft_state.craft_options.window_title.as_str()).with_visible(false);

        if let Some(window_size) = &craft_state.craft_options.window_size {
            window_attributes =
                window_attributes.with_inner_size(LogicalSize::new(window_size.width, window_size.height));
        }

        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            window_attributes.with_canvas(Some(canvas))
        };

        let window: Arc<Window> =
            Arc::from(event_loop.create_window(window_attributes).expect("Failed to create window."));
        info!("Created window");

        //craft_state.event_reducer.set_scale_factor(&window);

        let renderer_type = craft_state.craft_options.renderer;
        let window_copy = window.clone();

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    let renderer = craft_state.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Box<dyn Renderer> = renderer_type.create(window_copy).await;
                    renderer
                });
                craft_state.craft_app.on_resume(window, renderer, event_loop);
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let craft_state = &mut self.craft_state;

        CURRENT_WINDOW_ID.set(Some(window_id));
        // Create a new document if there is none for the current window.
        DOCUMENTS.with_borrow_mut(|docs| {
            if docs.get_document_by_window_id(window_id).is_none() {
                docs.add_document(window_id, Document::new());
            }
        });

        #[cfg(feature = "accesskit")]
        if let Some(accesskit_adapter) = &mut craft_state.craft_app.accesskit_adapter {
            accesskit_adapter.process_event(craft_state.craft_app.window.as_ref().unwrap(), &event);
        }

        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match craft_state.event_reducer.reduce(1.0, &event) {
                Some(WindowEventTranslation::Keyboard(keyboard_event)) => {
                    use ui_events::keyboard::{Key, NamedKey};
                    if keyboard_event.state.is_down() && matches!(keyboard_event.key, Key::Named(NamedKey::Escape)) {
                        event_loop.exit();
                    } else {
                        craft_state.craft_app.on_keyboard_input(keyboard_event);
                    }
                    return;
                }
                Some(WindowEventTranslation::Pointer(pointer_event)) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            craft_state.craft_app.on_pointer_button(pointer_button_update, false);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            craft_state.craft_app.on_pointer_button(pointer_button_update, true);
                        }
                        PointerEvent::Move(pointer_update) => {
                            craft_state.craft_app.on_pointer_moved(pointer_update);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            craft_state.craft_app.on_pointer_scroll(pointer_scroll_update);
                        },
                        PointerEvent::Gesture(_) => todo!()
                    }
                    return;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                craft_state.close_requested = true;
                craft_state.craft_app.on_close_requested();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                craft_state.craft_app.on_scale_factor_changed(scale_factor);
            }
            WindowEvent::Resized(new_size) => {
                craft_state.craft_app.on_resize(new_size);
            }
            WindowEvent::Ime(ime) => {
                craft_state.craft_app.on_ime(ime);
            }
            WindowEvent::RedrawRequested => {
                craft_state.craft_app.on_request_redraw();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        let craft_state = &mut self.craft_state;

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    craft_state.runtime.borrow_tokio_runtime().block_on(async {
                    while let Ok(message) = craft_state.winit_receiver.try_recv() {
                        match message {
                            InternalMessage::ResourceEvent(resource_event) => {
                                craft_state.craft_app.on_resource_event(resource_event);
                            }
                            #[cfg(target_arch = "wasm32")]
                            InternalMessage::RendererCreated(window, renderer) => {
                                craft_state.craft_app.on_resume(window, renderer);
                            }
                        }
                    }
                });
            } else {
                WASM_QUEUE.with_borrow_mut(|wasm_queue: &mut WasmQueue| {
                    wasm_queue.drain(|message| {
                        match message {
                            InternalMessage::ResourceEvent(resource_event) => {
                                craft_state.craft_app.on_resource_event(resource_event);
                            }
                            #[cfg(target_arch = "wasm32")]
                            InternalMessage::RendererCreated(window, renderer) => {
                                craft_state.craft_app.on_resume(window, renderer, event_loop);
                                if let Some(window) = craft_state.craft_app.window.as_ref() {
                                    window.request_redraw();
                                }
                            }
                        }
                    });
                });
            }
        }

        if craft_state.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
            return;
        }

        //self.craft_state.craft_app.window.clone().unwrap().request_redraw();

        // Switch to Poll mode if we are running animations.

/*        let has_active_animation = self.craft_state.craft_app.previous_animation_flags.has_active_animation();

        if has_active_animation {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
        event_loop.set_control_flow(ControlFlow::Poll);
        */
    }
}

impl CraftState {
    pub(crate) fn new(
        runtime: CraftRuntimeHandle,
        winit_receiver: Receiver<InternalMessage>,
        app_sender: Sender<InternalMessage>,
        craft_options: CraftOptions,
        craft_app: Box<App>,
    ) -> Self {
        Self {
            runtime,
            wait_cancelled: false,
            close_requested: false,
            winit_receiver,
            app_sender,
            craft_options,
            event_reducer: Default::default(),
            craft_app,
        }
    }
}
