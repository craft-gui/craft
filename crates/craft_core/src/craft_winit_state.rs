#[cfg(target_arch = "wasm32")]
use {
    std::cell::RefCell, std::ops::AddAssign, std::rc::Rc, wasm_bindgen::JsCast,
    winit::platform::web::WindowAttributesExtWeb,
};

#[cfg(feature = "vello_renderer")]
use crate::renderer::vello::VelloRenderer;

#[cfg(feature = "vello_cpu_renderer")]
use crate::renderer::vello_cpu::VelloCpuRenderer;

#[cfg(feature = "vello_hybrid_renderer")]
use crate::renderer::vello_hybrid::VelloHybridRenderer;

use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::events::{KeyboardInput, MouseWheel, PointerButton, PointerMoved};
use crate::geometry::Size;
use crate::renderer::blank_renderer::BlankRenderer;
use crate::renderer::renderer::Renderer;
use crate::{CraftOptions, CraftRuntime, RendererType, WAIT_TIME};
use craft_logging::info;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use web_time as time;
#[cfg(not(target_arch = "wasm32"))]
use std::time;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use winit::dpi::LogicalSize;

/// Stores state related to Winit.
///
/// Forwards most events to the main Craft Event Loop.
pub(crate) struct CraftWinitState {
    #[cfg(not(target_arch = "wasm32"))]
    id: u64,
    #[cfg(target_arch = "wasm32")]
    id: Rc<RefCell<u64>>,
    #[allow(dead_code)]
    runtime: CraftRuntime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<dyn Window>>,
    #[allow(dead_code)]
    winit_receiver: Receiver<AppMessage>,
    app_sender: Sender<AppMessage>,
    craft_options: CraftOptions,
}

impl ApplicationHandler for CraftWinitState {
    fn new_events(&mut self, _event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default().with_title(self.craft_options.window_title.as_str());
        
        if let Some(window_size) = &self.craft_options.window_size {
            window_attributes = window_attributes.with_surface_size(LogicalSize::new(window_size.width, window_size.height));
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

        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(window_attributes).expect("Failed to create window."));
        info!("Created window");

        window.set_ime_allowed(true);

        self.window = Some(window.clone());
        info!("Creating renderer");
        info!("Using {} renderer.", self.craft_options.renderer);

        let renderer_type = self.craft_options.renderer;
        let window_copy = window.clone();

        let renderer_future: Pin<Box<dyn Future<Output = Box<dyn Renderer + Send>>>> = Box::pin(async move {
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

        #[cfg(target_arch = "wasm32")]
        {
            let mut tx = self.app_sender.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let renderer = renderer_future.await;

                info!("Created renderer");
                tx.send(AppMessage::new(0, InternalMessage::Resume(window.clone(), Some(renderer))))
                    .await
                    .expect("Sending app message failed");
                window.request_redraw();
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = self.runtime.borrow_tokio_runtime().block_on(renderer_future);
            info!("Created renderer");
            self.send_message(InternalMessage::Resume(window, Some(renderer)), true);
        }
    }

    fn window_event(&mut self, _event_loop: &dyn ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::CloseRequested => {
                self.send_message(InternalMessage::Close, true);
                self.close_requested = true;
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.send_message(InternalMessage::ModifiersChanged(modifiers), true);
            }
            WindowEvent::PointerButton {
                device_id,
                state,
                position,
                button,
                primary,
            } => {
                let event = PointerButton::new(device_id, state, position, button, primary);
                self.send_message(InternalMessage::PointerButton(event), false);
            }
            WindowEvent::PointerMoved {
                device_id,
                position,
                source,
                primary,
            } => {
                self.send_message(
                    InternalMessage::PointerMoved(PointerMoved::new(device_id, position, source, primary)),
                    true,
                );
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let event = MouseWheel::new(device_id, delta, phase);
                self.send_message(InternalMessage::MouseWheel(event), true);
            }
            WindowEvent::SurfaceResized(new_size) => {
                self.send_message(InternalMessage::Resize(new_size), true);
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                self.send_message(
                    InternalMessage::KeyboardInput(KeyboardInput::new(device_id, event, is_synthetic)),
                    true,
                );
            }
            WindowEvent::Ime(ime) => {
                self.send_message(InternalMessage::Ime(ime), true);
            }
            WindowEvent::RedrawRequested => {
                // We want to do any window operations within the main thread.
                // On some operating systems, the window is not thread-safe.
                let window = self.window.clone().unwrap();
                let scale_factor = window.scale_factor();
                let surface_size = Size::new(window.surface_size().width as f32, window.surface_size().height as f32);

                self.send_message(InternalMessage::RequestRedraw(scale_factor, surface_size), true);
                window.pre_present_notify();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            //self.window.as_ref().unwrap().request_redraw();
        }

        if self.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.runtime.borrow_tokio_runtime().block_on(async {
                tokio::task::yield_now().await;
            })
        }

        self.send_message(InternalMessage::ProcessUserEvents, false);

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
    }
}

impl CraftWinitState {
    pub(crate) fn new(
        runtime: CraftRuntime,
        winit_receiver: Receiver<AppMessage>,
        app_sender: Sender<AppMessage>,
        craft_options: CraftOptions,
    ) -> Self {
        Self {
            id: Default::default(),
            runtime,
            request_redraw: false,
            wait_cancelled: false,
            close_requested: false,
            window: None,
            winit_receiver,
            app_sender,
            craft_options,
        }
    }

    fn send_message(&mut self, message: InternalMessage, blocking: bool) {
        let is_close_message = matches!(message, InternalMessage::Close);

        let app_message = AppMessage {
            id: self.get_id(),
            blocking,
            data: message,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.runtime.borrow_tokio_runtime().block_on(async {
                let result: Result<(), SendError<AppMessage>> = self.app_sender.send(app_message).await;
                if !is_close_message {
                    result.expect("Failed to send app message");
                }

                if blocking {
                    if let Some(response) = self.winit_receiver.recv().await {
                        if let InternalMessage::Confirmation = response.data {
                            assert_eq!(response.id, self.id, "Expected response message with id {}", self.id);
                        } else {
                            panic!("Expected response message, but response was something else");
                        }
                    }
                }
            });
            self.id += 1;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut tx = self.app_sender.clone();
            wasm_bindgen_futures::spawn_local(async move {
                tx.send(app_message).await.expect("send failed");
            });
            self.id.borrow_mut().add_assign(1);
        }
    }

    fn get_id(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            *self.id.borrow()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.id
        }
    }
}
