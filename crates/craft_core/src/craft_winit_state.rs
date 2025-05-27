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

use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::geometry::Size;
use crate::renderer::blank_renderer::BlankRenderer;
use crate::renderer::renderer::Renderer;
use crate::{App, CraftOptions, CraftRuntime, RendererType, WAIT_TIME};
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

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, NodeId, Role, TreeUpdate};
use accesskit_winit::Adapter;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc::error::SendError;
use ui_events::pointer::PointerEvent;
use ui_events::UiEvent;
use ui_events_winit::WindowEventReducer;
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
    window: Option<Arc<Window>>,
    #[allow(dead_code)]
    winit_receiver: Receiver<AppMessage>,
    app_sender: Sender<AppMessage>,
    craft_options: CraftOptions,
    event_reducer: WindowEventReducer,
    accesskit_adapter: Option<Adapter>,
    craft_app: Option<Box<App>>,
}

struct CraftActivationHandler {
    tree_update: Option<TreeUpdate>,
}

impl ActivationHandler for CraftActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        /*let mut window_node = accesskit::Node::new(Role::Window);
        let mut button_node = accesskit::Node::new(Role::Button);
        button_node.set_bounds(accesskit::Rect::new(0.0, 0.0, 100.0, 100.0));
        button_node.set_description("silly button".to_string());
        button_node.set_label("Button".to_string());
        button_node.add_action(Action::Focus);
        button_node.add_action(Action::Click);

        let tree = Tree::new(NodeId(0));
        window_node.set_children(vec![NodeId(1)]);
        let tree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), window_node),
                (NodeId(1), button_node)
            ],
            tree: Some(tree),
            focus: NodeId(1),
        };
*/
        Some(self.tree_update.take().unwrap())
    }
}

struct CraftAccessHandler {}

impl ActionHandler for CraftAccessHandler {
    fn do_action(&mut self, request: ActionRequest) {
        println!("Action requested: {:?}", request);
    }
}

struct CraftDeactivationHandler {}

impl DeactivationHandler for CraftDeactivationHandler {
    fn deactivate_accessibility(&mut self) {}
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

        window.set_ime_allowed(true);

        let action_handler = CraftAccessHandler {};
        let deactivation_handler = CraftDeactivationHandler {};

        let scale_factor = window.scale_factor();
        let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

        let mut app = self.craft_app.take().unwrap();
        app.on_request_redraw(scale_factor, surface_size);
        println!("Size: {:?}", surface_size);

        let tree = accesskit::Tree {
            root: NodeId(0),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: NodeId(0),
        };
        app.user_tree.element_tree.as_mut().unwrap().compute_accessibility_tree(&mut tree_update, None);

        tree_update.nodes[0].1.set_role(Role::Window);
        println!("{:?}", tree_update);

        let craft_activation_handler = CraftActivationHandler {
            tree_update: Some(tree_update),
        };

        self.accesskit_adapter = Some(Adapter::with_direct_handlers(event_loop, &window, craft_activation_handler, action_handler, deactivation_handler));
        self.send_message(InternalMessage::TakeApp(app), false);

        window.set_visible(true);

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
            let tx = self.app_sender.clone();
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            //println!("Processing event: {:?}", event);
            accesskit_adapter.process_event(self.window.as_ref().unwrap(), &event);
        }

        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match self.event_reducer.reduce(&event) {
                UiEvent::Keyboard(k) => {
                    use ui_events::keyboard::{Key, NamedKey};
                    if k.state.is_down() && matches!(k.key, Key::Named(NamedKey::Escape)) {
                        event_loop.exit();
                    } else {
                        self.send_message(InternalMessage::KeyboardInput(k), false);
                    }
                    return;
                }
                UiEvent::Pointer(pointer_event) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            self.send_message(InternalMessage::PointerButtonDown(pointer_button_update), false);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            self.send_message(InternalMessage::PointerButtonUp(pointer_button_update), false);
                        }
                        PointerEvent::Move(pointer_update) => {
                            self.send_message(InternalMessage::PointerMoved(pointer_update), false);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            self.send_message(InternalMessage::PointerScroll(pointer_scroll_update), true);
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::CloseRequested => {
                self.send_message(InternalMessage::Close, true);
                self.close_requested = true;
            }
            WindowEvent::Resized(new_size) => {
                self.send_message(InternalMessage::Resize(new_size), true);
            }
            WindowEvent::Ime(ime) => {
                self.send_message(InternalMessage::Ime(ime), true);
            }
            WindowEvent::RedrawRequested => {
                // We want to do any window operations within the main thread.
                // On some operating systems, the window is not thread-safe.
                let window = self.window.clone().unwrap();
                let scale_factor = window.scale_factor();
                let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

                self.send_message(InternalMessage::RequestRedraw(scale_factor, surface_size), true);
                window.pre_present_notify();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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
        craft_app: Box<App>
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
            event_reducer: Default::default(),
            accesskit_adapter: None,
            craft_app: Some(craft_app),
        }
    }

    fn send_message(&mut self, message: InternalMessage, blocking: bool) {
        #[cfg(not(target_arch = "wasm32"))]
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
            let tx = self.app_sender.clone();
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
