use std::sync::Arc;
use winit::application::ApplicationHandler;

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    winit::platform::web::WindowAttributesExtWebSys,
};  


#[cfg(target_arch = "wasm32")]
use {craft::resource_manager::wasm_queue::WasmQueue, craft::resource_manager::wasm_queue::WASM_QUEUE};

#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;
use crate::wgpu_triangle::draw_gui_texture_and_canvas;
use craft::craft_winit_state::CraftState;
use craft::elements::{Canvas, Element};
use craft::events::internal::InternalMessage;
use craft::events::ui_events::keyboard::{Key, NamedKey};
use craft::events::ui_events::pointer::PointerEvent;
use craft::events::ui_events::UiEvent;
use craft::events::EventDispatchType;
use craft_renderer::vello::VelloRenderer;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowAttributes, WindowId};

pub(crate) struct CraftWinitState {
    rotation: f32,
    pub start_time: time::Instant,
    craft_state: CraftState,
}

impl CraftWinitState {
    pub(crate) fn new(craft_state: CraftState) -> Self {
        Self {
            rotation: 0.0,
            start_time: time::Instant::now(),
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

        craft_state.event_reducer.set_scale_factor(&window);

        let _renderer_type = craft_state.craft_options.renderer;
        let window_copy = window.clone();


        let render_vello_to_texture = true;
        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let renderer = craft_state.runtime.borrow_tokio_runtime().block_on(
                    async {
                        Box::new(VelloRenderer::new(window_copy, render_vello_to_texture).await)
                    }
                );
                craft_state.craft_app.on_resume(window, renderer, event_loop);
            } else {
                let app_sender = craft_state.app_sender.clone();
                let window_copy_2 = window_copy.clone();
                craft_state.runtime.spawn(async move {
                    let renderer: Box<dyn Renderer> = Box::new(VelloRenderer::new(window_copy, render_vello_to_texture).await);
                    app_sender
                        .send(InternalMessage::RendererCreated(window_copy_2, renderer))
                        .await
                        .expect("Failed to send RendererCreated message");
                });
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let craft_state = &mut self.craft_state;

        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match craft_state.event_reducer.reduce(&event) {
                UiEvent::Keyboard(keyboard_event) => {
                    if keyboard_event.state.is_down() && matches!(keyboard_event.key, Key::Named(NamedKey::Escape)) {
                        event_loop.exit();
                    } else {
                        craft_state.craft_app.on_keyboard_input(keyboard_event);
                    }
                    return;
                }
                UiEvent::Pointer(pointer_event) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            craft_state.craft_app.on_pointer_button(pointer_button_update, false, EventDispatchType::Bubbling);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            craft_state.craft_app.on_pointer_button(pointer_button_update, true, EventDispatchType::Bubbling);
                        }
                        PointerEvent::Move(pointer_update) => {
                            craft_state.craft_app.on_pointer_moved(pointer_update);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            craft_state.craft_app.on_pointer_scroll(pointer_scroll_update);
                        }
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
                self.craft_state.craft_app.on_request_redraw();
                
                let canvas = if let Some(root) = self.craft_state.craft_app.user_tree.element_tree.as_mut() {
                    let mut found = None;
                    for ele in root.pre_order_iter() {
                        if let Some(canvas) = ele.as_any().downcast_ref::<Canvas>() {
                            found = Some(canvas);
                            break;
                        }
                    };

                    found
                } else {
                    None
                };

                if let Some(renderer) = self.craft_state.craft_app.renderer.as_mut() && let Some(canvas) = canvas &&
                    let Some(vello_renderer) = renderer.as_any_mut().downcast_mut::<VelloRenderer>() {
                    let scale_factor = self.craft_state.craft_app.window_context.effective_scale_factor() as f32;

                    let size = canvas.computed_box_transformed().padding_rectangle_size();
                    let position = canvas.computed_box_transformed().padding_rectangle_position();

                    let elapsed = self.start_time.elapsed().as_secs_f32();
                    let angle = elapsed % (2.0 * std::f32::consts::PI);
                    self.rotation = angle;

                    draw_gui_texture_and_canvas(vello_renderer,
                                                position.x as f32 * scale_factor,
                                                position.y as f32 * scale_factor,
                                                size.width * scale_factor,
                                                size.height * scale_factor,
                                                self.rotation
                    );
                }
                
                if let Some(window) = &mut self.craft_state.craft_app.window {
                    window.request_redraw();
                }
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
                            InternalMessage::GotUserMessage(user_message) => {
                               craft_state.craft_app.on_user_message(user_message);
                            }
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
                            InternalMessage::GotUserMessage(user_message) => {
                                craft_state.craft_app.on_user_message(user_message);
                            }
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
            event_loop.exit();
            return;
        }

        if !craft_state.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }
}