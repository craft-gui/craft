#[cfg(target_arch = "wasm32")]
use {
    winit::platform::web::WindowAttributesExtWeb,
    
    wasm_bindgen::JsCast,

    std::rc::Rc,
    std::ops::AddAssign,
    std::cell::RefCell,
};

#[cfg(feature = "vello_renderer")]
use crate::renderer::vello::VelloRenderer;

use crate::renderer::blank_renderer::BlankRenderer;
use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::events::{KeyboardInput, MouseWheel, PointerButton, PointerMoved};
use crate::renderer::renderer::Renderer;
use crate::geometry::Size;
use oku_logging::info;
use crate::{OkuOptions, OkuRuntime, RendererType, WAIT_TIME};

#[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))]
use crate::renderer::softbuffer::SoftwareRenderer;

#[cfg(feature = "wgpu_renderer")]
use crate::renderer::wgpu::WgpuRenderer;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use web_time as time;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
use std::time;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

/// Stores state relate to Winit.
///
/// Forwards most events to the main Oku Event Loop.
pub(crate) struct OkuWinitState {
    #[cfg(not(target_arch = "wasm32"))]
    id: u64,
    #[cfg(target_arch = "wasm32")]
    id: Rc<RefCell<u64>>,
    #[allow(dead_code)]
    runtime: OkuRuntime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<dyn Window>>,
    #[allow(dead_code)]
    winit_receiver: Receiver<AppMessage>,
    app_sender: Sender<AppMessage>,
    oku_options: OkuOptions,
}

impl ApplicationHandler for OkuWinitState {
    fn new_events(&mut self, _event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_title(self.oku_options.window_title.as_str());

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

        self.window = Some(window.clone());
        info!("Creating renderer");
        info!("Using {} renderer.", self.oku_options.renderer);

        let renderer_type = self.oku_options.renderer;
        let window_copy = window.clone();

        let renderer_future: Pin<Box<dyn Future<Output = Box<dyn Renderer + Send>>>> = Box::pin(async move {
            let renderer: Box<dyn Renderer + Send> = match renderer_type {
                #[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))]
                RendererType::Software => Box::new(SoftwareRenderer::new(window_copy)),
                #[cfg(feature = "wgpu_renderer")]
                RendererType::Wgpu => Box::new(WgpuRenderer::new(window_copy).await),
                #[cfg(feature = "vello_renderer")]
                RendererType::Vello => Box::new(VelloRenderer::new(window_copy).await),
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
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            //self.window.as_ref().unwrap().request_redraw();
        }

        #[cfg(not(target_arch = "wasm32"))] {
            self.runtime.borrow_tokio_runtime().block_on(async {
                tokio::task::yield_now().await;
            })
        }

        self.send_message(InternalMessage::ProcessUserEvents, false);

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }

        if self.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
        }
    }
}

impl OkuWinitState {
    pub(crate) fn new(
        runtime: OkuRuntime,
        winit_receiver: Receiver<AppMessage>,
        app_sender: Sender<AppMessage>,
        oku_options: OkuOptions,
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
            oku_options,
        }
    }

    fn send_message(&mut self, message: InternalMessage, blocking: bool) {
        let app_message = AppMessage {
            id: self.get_id(),
            blocking,
            data: message,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.runtime.borrow_tokio_runtime().block_on(async {
                self.app_sender.send(app_message).await.expect("send failed");
                if blocking {
                    if let Some(response) = self.winit_receiver.recv().await {
                        if let InternalMessage::Confirmation = response.data {
                            assert_eq!(response.id, self.id, "Expected response message with id {}", self.id);
                        } else {
                            panic!("Expected response message, but response was something else");
                        }
                    } else {
                        panic!("Expected response message, but response was empty");
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
