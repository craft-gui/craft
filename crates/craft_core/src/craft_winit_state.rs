#[cfg(target_arch = "wasm32")]
use {
    std::cell::RefCell, std::ops::AddAssign, std::rc::Rc, wasm_bindgen::JsCast,
    winit::platform::web::WindowAttributesExtWebSys,
};

#[cfg(feature = "vello_renderer")]
use crate::renderer::vello::VelloRenderer;

#[cfg(feature = "vello_cpu_renderer")]
use crate::renderer::vello_cpu::VelloCpuRenderer;

#[cfg(feature = "vello_hybrid_renderer")]
use crate::renderer::vello_hybrid::VelloHybridRenderer;

#[cfg(target_arch = "wasm32")]
use {
    crate::resource_manager::wasm_queue::WASM_QUEUE,
    crate::resource_manager::wasm_queue::WasmQueue,
};

use crate::events::internal::InternalMessage;
use crate::renderer::blank_renderer::BlankRenderer;
use crate::renderer::renderer::Renderer;
use crate::{App, CraftOptions, RendererType, WAIT_TIME};
use craft_logging::info;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::craft_runtime::CraftRuntimeHandle;
use std::sync::Arc;
use ui_events::pointer::{PointerEvent};
use ui_events::UiEvent;
use ui_events_winit::WindowEventReducer;
use winit::dpi::LogicalSize;
use crate::events::EventDispatchType;

/// Stores state related to Winit.
///
/// Forwards most events to the main Craft Event Loop.
pub(crate) struct CraftWinitState {
    #[allow(dead_code)]
    runtime: CraftRuntimeHandle,
    wait_cancelled: bool,
    close_requested: bool,
    #[allow(dead_code)]
    winit_receiver: Receiver<InternalMessage>,
    #[allow(dead_code)]
    app_sender: Sender<InternalMessage>,
    craft_options: CraftOptions,
    event_reducer: WindowEventReducer,
    craft_app: Box<App>,
}

impl ApplicationHandler for CraftWinitState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes =
            WindowAttributes::default().with_title(self.craft_options.window_title.as_str()).with_visible(false);

        if let Some(window_size) = &self.craft_options.window_size {
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

        let renderer_type = self.craft_options.renderer;
        let window_copy = window.clone();

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    let renderer = self.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Box<dyn Renderer + Send> = match renderer_type {
                        #[cfg(feature = "vello_renderer")]
                        RendererType::Vello => Box::new(VelloRenderer::new(window_copy).await),
                        #[cfg(feature = "vello_cpu_renderer")]
                        RendererType::VelloCPU => Box::new(VelloCpuRenderer::new(window_copy)),
                        #[cfg(feature = "vello_hybrid_renderer")]
                        RendererType::VelloHybrid => Box::new(VelloHybridRenderer::new(window_copy).await),
                        RendererType::Blank => Box::new(BlankRenderer),
                    };
                    renderer
                });
                self.craft_app.on_resume(window, renderer, event_loop);
            } else {
                let app_sender = self.app_sender.clone();
                let window_copy_2 = window_copy.clone();
                self.runtime.spawn(async move {
                    let renderer: Box<dyn Renderer> = match renderer_type {
                        #[cfg(feature = "vello_renderer")]
                        RendererType::Vello => Box::new(VelloRenderer::new(window_copy).await),
                        #[cfg(feature = "vello_cpu_renderer")]
                        RendererType::VelloCPU => Box::new(VelloCpuRenderer::new(window_copy)),
                        #[cfg(feature = "vello_hybrid_renderer")]
                        RendererType::VelloHybrid => Box::new(VelloHybridRenderer::new(window_copy).await),
                        RendererType::Blank => Box::new(BlankRenderer),
                    };
                    app_sender
                        .send(InternalMessage::RendererCreated(window_copy_2, renderer))
                        .await
                        .expect("Failed to send RendererCreated message");
                });
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(accesskit_adapter) = &mut self.craft_app.accesskit_adapter {
            accesskit_adapter.process_event(self.craft_app.window.as_ref().unwrap(), &event);
        }

        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match self.event_reducer.reduce(&event) {
                UiEvent::Keyboard(keyboard_event) => {
                    use ui_events::keyboard::{Key, NamedKey};
                    if keyboard_event.state.is_down() && matches!(keyboard_event.key, Key::Named(NamedKey::Escape)) {
                        event_loop.exit();
                    } else {
                        self.craft_app.on_keyboard_input(keyboard_event);
                    }
                    return;
                }
                UiEvent::Pointer(pointer_event) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            self.craft_app.on_pointer_button(pointer_button_update, false, EventDispatchType::Bubbling);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            self.craft_app.on_pointer_button(pointer_button_update, true, EventDispatchType::Bubbling);
                        }
                        PointerEvent::Move(pointer_update) => {
                            self.craft_app.on_pointer_moved(pointer_update);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                           self.craft_app.on_pointer_scroll(pointer_scroll_update);
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                self.craft_app.on_close_requested();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                ..
            } => {
                self.craft_app.on_scale_factor_changed(scale_factor);
            }
            WindowEvent::Resized(new_size) => {
                self.craft_app.on_resize(new_size);
            }
            WindowEvent::Ime(ime) => {
                self.craft_app.on_ime(ime);
            }
            WindowEvent::RedrawRequested => {
                self.craft_app.on_request_redraw();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    self.runtime.borrow_tokio_runtime().block_on(async {
                    while let Ok(message) = self.winit_receiver.try_recv() {
                        match message {
                            InternalMessage::GotUserMessage(user_message) => {
                               self.craft_app.on_user_message(user_message);
                            }
                            InternalMessage::ResourceEvent(resource_event) => {
                                self.craft_app.on_resource_event(resource_event);
                            }
                            #[cfg(target_arch = "wasm32")]
                            InternalMessage::RendererCreated(window, renderer) => {
                                self.craft_app.on_resume(window, renderer);
                            }
                        }
                    }
                });
            } else {
                WASM_QUEUE.with_borrow_mut(|wasm_queue: &mut WasmQueue| {
                    wasm_queue.drain(|message| {
                        match message {
                            InternalMessage::GotUserMessage(user_message) => {
                                self.craft_app.on_user_message(user_message);
                            }
                            InternalMessage::ResourceEvent(resource_event) => {
                                self.craft_app.on_resource_event(resource_event);
                            }
                            #[cfg(target_arch = "wasm32")]
                            InternalMessage::RendererCreated(window, renderer) => {
                                self.craft_app.on_resume(window, renderer, event_loop);
                                if let Some(window) = self.craft_app.window.as_ref() {
                                    window.request_redraw();
                                }
                            }
                            _ => {}
                        }
                    });
                });
            }
        }

    if self.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
            return;
        }

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
    }
}

impl CraftWinitState {
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

    /*fn wait_for_redraw(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.runtime.maybe_block_on(async {
                if let Some(app_message) = self.winit_receiver.recv().await {
                    if let InternalMessage::RequestWinitRedraw(redraw) = app_message {
                        if redraw {
                            if let Some(window) = self.window.as_ref() {
                                window.request_redraw();
                            }
                        }
                    } else {
                        panic!("Expected RequestWinitRedraw message, but received something else");
                    }
                }
            })
        }
    }*/
}
