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

use crate::events::internal::InternalMessage;
use crate::geometry::Size;
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
use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate,
};
use accesskit_winit::Adapter;
use std::sync::Arc;
use ui_events::pointer::{PointerButton, PointerButtonUpdate, PointerEvent, PointerInfo, PointerState, PointerType};
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
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<Window>>,
    #[allow(dead_code)]
    winit_receiver: Receiver<InternalMessage>,
    app_sender: Sender<InternalMessage>,
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

#[cfg(not(target_arch = "wasm32"))]
struct CraftAccessHandler {
    #[cfg(not(target_arch = "wasm32"))]
    runtime_handle: CraftRuntimeHandle,
    app_sender: Sender<InternalMessage>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ActionHandler for CraftAccessHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let ass = self.app_sender.clone();
        self.runtime_handle.spawn(async move {
            match request.action {
                Action::Click => {}
                Action::Focus => {}
                Action::Blur => {}
                Action::Collapse => {}
                Action::Expand => {}
                Action::CustomAction => {}
                Action::Decrement => {}
                Action::Increment => {}
                Action::HideTooltip => {}
                Action::ShowTooltip => {}
                Action::ReplaceSelectedText => {}
                Action::ScrollBackward => {}
                Action::ScrollDown => {}
                Action::ScrollForward => {}
                Action::ScrollLeft => {}
                Action::ScrollRight => {}
                Action::ScrollUp => {}
                Action::ScrollIntoView => {}
                Action::ScrollToPoint => {}
                Action::SetScrollOffset => {}
                Action::SetTextSelection => {}
                Action::SetSequentialFocusNavigationStartingPoint => {}
                Action::SetValue => {}
                Action::ShowContextMenu => {}
            }
            println!("Action requested: {:?}", request);
            ass.send(InternalMessage::PointerButtonUp(PointerButtonUpdate {
                button: Some(PointerButton::Primary),
                pointer: PointerInfo {
                    pointer_id: None,
                    persistent_device_id: None,
                    pointer_type: PointerType::Mouse,
                },
                state: PointerState {
                    time: 0,
                    position: Default::default(),
                    buttons: Default::default(),
                    modifiers: Default::default(),
                    count: 0,
                    contact_geometry: Default::default(),
                    orientation: Default::default(),
                    pressure: 0.0,
                    tangential_pressure: 0.0,
                },
            }, EventDispatchType::Accesskit(request.target.0))).await.expect("TODO: panic message");
        });
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


        #[cfg(not(target_arch = "wasm32"))]
        let action_handler = CraftAccessHandler {
            runtime_handle: self.runtime.clone(),
            app_sender: self.app_sender.clone(),
        };
        let deactivation_handler = CraftDeactivationHandler {};

        let scale_factor = window.scale_factor();
        let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

        let mut app = self.craft_app.take().unwrap();
        app.on_request_redraw(scale_factor, surface_size);

        let tree_update = app.compute_accessibility_tree();

        let craft_activation_handler = CraftActivationHandler {
            tree_update: Some(tree_update),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.accesskit_adapter = Some(Adapter::with_direct_handlers(
                event_loop,
                &window,
                craft_activation_handler,
                action_handler,
                deactivation_handler,
            ));
        }

        let app_sender = self.app_sender.clone();
        self.runtime.maybe_block_on(async move {
            app_sender.send(InternalMessage::TakeApp(app)).await.expect("Failed to send TakeApp message");
        });

        window.set_visible(true);

        self.window = Some(window.clone());
        info!("Creating renderer");
        info!("Using {} renderer.", self.craft_options.renderer);

        let renderer_type = self.craft_options.renderer;
        let window_copy = window.clone();

        let tx = self.app_sender.clone();
        self.runtime.maybe_block_on(async move {
            #[cfg(target_arch = "wasm32")]
            let renderer: Box<dyn Renderer> = match renderer_type {
                #[cfg(feature = "vello_renderer")]
                RendererType::Vello => Box::new(VelloRenderer::new(window_copy).await),
                #[cfg(feature = "vello_cpu_renderer")]
                RendererType::VelloCPU => Box::new(VelloCpuRenderer::new(window_copy)),
                #[cfg(feature = "vello_hybrid_renderer")]
                RendererType::VelloHybrid => Box::new(VelloHybridRenderer::new(window_copy).await),
                RendererType::Blank => Box::new(BlankRenderer),
            };

            #[cfg(not(target_arch = "wasm32"))]
            let renderer: Box<dyn Renderer + Send> = match renderer_type {
                #[cfg(feature = "vello_renderer")]
                RendererType::Vello => Box::new(VelloRenderer::new(window_copy).await),
                #[cfg(feature = "vello_cpu_renderer")]
                RendererType::VelloCPU => Box::new(VelloCpuRenderer::new(window_copy)),
                #[cfg(feature = "vello_hybrid_renderer")]
                RendererType::VelloHybrid => Box::new(VelloHybridRenderer::new(window_copy).await),
                RendererType::Blank => Box::new(BlankRenderer),
            };

            info!("Created renderer");
            tx.send(InternalMessage::Resume(window.clone(), Some(renderer))).await.expect("Sending app message failed");
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            accesskit_adapter.process_event(self.window.as_ref().unwrap(), &event);
        }

        let app_sender = self.app_sender.clone();
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
                        self.runtime.spawn(async move {
                            app_sender.send(InternalMessage::KeyboardInput(k)).await.unwrap();
                        });
                    }
                    return;
                }
                UiEvent::Pointer(pointer_event) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            self.runtime.spawn(async move {
                                app_sender.send(InternalMessage::PointerButtonDown(pointer_button_update, EventDispatchType::Bubbling)).await.unwrap();
                            });
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            self.runtime.spawn(async move {
                                app_sender.send(InternalMessage::PointerButtonUp(pointer_button_update, EventDispatchType::Bubbling)).await.unwrap();
                            });
                        }
                        PointerEvent::Move(pointer_update) => {
                            self.runtime.spawn(async move {
                                app_sender.send(InternalMessage::PointerMoved(pointer_update)).await.unwrap();
                            });
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            self.runtime.spawn(async move {
                                app_sender.send(InternalMessage::PointerScroll(pointer_scroll_update)).await.unwrap();
                            });
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
                self.close_requested = true;
                self.runtime.spawn(async move {
                    app_sender.send(InternalMessage::Close).await.unwrap();
                });
            }
            WindowEvent::Resized(new_size) => {
                self.runtime.spawn(async move {
                    app_sender.send(InternalMessage::Resize(new_size)).await.unwrap();
                });
            }
            WindowEvent::Ime(ime) => {
                self.runtime.spawn(async move {
                    app_sender.send(InternalMessage::Ime(ime)).await.unwrap();
                });
            }
            WindowEvent::RedrawRequested => {
                // Ideally redraws should be completed synchronously.
                // However, on wasm32, we cannot block the event loop,

                let window = self.window.clone().unwrap();
                let scale_factor = window.scale_factor();
                let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

                #[cfg(not(target_arch = "wasm32"))]
                self.runtime.maybe_block_on(async {
                    app_sender.send(InternalMessage::RequestRedraw(scale_factor.clone(), surface_size)).await.unwrap();

                    if let Some(app_message) = self.winit_receiver.recv().await {
                        if let InternalMessage::AccesskitTreeUpdate(tree_update) = app_message {
                            info!("Received accessibility tree update:");
                            self.accesskit_adapter.as_mut().unwrap().update_if_active(|| tree_update);
                        } else {
                            panic!("Expected accessibility tree update, but received something else");
                        }
                    }
                });
                #[cfg(target_arch = "wasm32")]
                {
                    self.runtime.spawn(async move {
                        app_sender.send(InternalMessage::RequestRedraw(scale_factor, surface_size)).await.unwrap();
                    });
                }
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

        let app_sender = self.app_sender.clone();
        self.runtime.spawn(async move {
            app_sender.send(InternalMessage::ProcessUserEvents).await.unwrap();
        });

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
}
