#[cfg(not(target_arch = "wasm32"))]
use std::time;

use craft_logging::info;
use craft_primitives::geometry::Size;
use craft_runtime::{CraftRuntimeHandle, Receiver, Sender};
use ui_events::pointer::PointerEvent;
use ui_events_winit::{WindowEventReducer, WindowEventTranslation};
#[cfg(target_arch = "wasm32")]
use web_time as time;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;
#[cfg(target_arch = "wasm32")]
use {crate::wasm_queue::WASM_QUEUE, crate::wasm_queue::WasmQueue};
#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, winit::platform::web::WindowAttributesExtWebSys};

use crate::CraftOptions;
use crate::app::{App, CURRENT_WINDOW_ID, DOCUMENTS, WINDOW_MANAGER};
use crate::document::Document;
use crate::events::internal::InternalMessage;

const WAIT_TIME: time::Duration = time::Duration::from_millis(15);

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
        Self { craft_state }
    }
}

impl ApplicationHandler for CraftWinitState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.craft_state.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.craft_state.craft_app.on_resume(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.craft_state.craft_app.on_suspended(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window: Option<crate::elements::Window> =
            WINDOW_MANAGER.with_borrow_mut(|window_manager| window_manager.get_window_by_id(window_id));

        let window = if let Some(window) = window { window } else { return };

        let craft_state = &mut self.craft_state;

        CURRENT_WINDOW_ID.set(Some(window_id));
        // Create a new document if there is none for the current window.
        DOCUMENTS.with_borrow_mut(|docs| {
            if docs.get_document_by_window_id(window_id).is_none() {
                docs.add_document(window_id, Document::new());
            }
        });

        /*#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        if let Some(accesskit_adapter) = &mut craft_state.craft_app.accesskit_adapter {
            accesskit_adapter.process_event(craft_state.craft_app.window.as_ref().unwrap(), &event);
        }*/

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
                        craft_state.craft_app.on_keyboard_input(window, keyboard_event);
                    }
                    return;
                }
                Some(WindowEventTranslation::Pointer(pointer_event)) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            craft_state
                                .craft_app
                                .on_pointer_button(window, pointer_button_update, false);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            craft_state
                                .craft_app
                                .on_pointer_button(window, pointer_button_update, true);
                        }
                        PointerEvent::Move(pointer_update) => {
                            craft_state.craft_app.on_pointer_moved(window, pointer_update);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            craft_state.craft_app.on_pointer_scroll(window, pointer_scroll_update);
                        }
                        PointerEvent::Gesture(_) => todo!(),
                    }
                    return;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                    window_manager.close_window(&window);
                    if window_manager.is_empty() {
                        craft_state.close_requested = true;
                        craft_state.craft_app.on_close_requested();
                    }
                });
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                craft_state.craft_app.on_scale_factor_changed(window, scale_factor);
            }
            WindowEvent::Resized(new_size) => {
                let new_size = Size::<f32> {
                    width: new_size.width as f32,
                    height: new_size.height as f32,
                };
                craft_state.craft_app.on_resize(window, new_size);
            }
            WindowEvent::Ime(ime) => {
                craft_state.craft_app.on_ime(window, ime);
            }
            WindowEvent::RedrawRequested => {
                craft_state.craft_app.on_request_redraw(window);
            }
            WindowEvent::Moved(_) => {
                craft_state.craft_app.on_move(window);
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

        craft_state.craft_app.on_about_to_wait(event_loop);

        if craft_state.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
            return;
        }

        // Switch to Poll mode if we are running animations.

        let has_active_animation = self
            .craft_state
            .craft_app
            .previous_animation_flags
            .has_active_animation();

        if has_active_animation {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
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
