//! Integration with the winit event loop.

use retgui_logging::info;
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use std::time::Instant;

use retgui_primitives::geometry::Size;

use retgui_runtime::{Job, Receiver, RetGuiRuntimeHandle, Sender, pop_gui_thread_work, push_gui_thread_work};

use ui_events::pointer::PointerEvent;

use ui_events_winit::{WindowEventReducer, WindowEventTranslation};

#[cfg(target_arch = "wasm32")]
use web_time as time;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use crate::RetGuiOptions;
use crate::app::{App, WINDOW_MANAGER, dequeue_window_event};
use crate::elements::Window;
use crate::events::internal::InternalMessage;
#[cfg(target_arch = "wasm32")]
use {crate::wasm_queue::WASM_QUEUE, crate::wasm_queue::WasmQueue};

const WAIT_TIME: time::Duration = time::Duration::from_millis(10);

/// Stores state related to Winit.
///
/// Forwards most events to the main RetGui Event Loop.
pub struct RetGuiState {
    #[allow(dead_code)]
    pub runtime: RetGuiRuntimeHandle,
    pub wait_cancelled: bool,
    pub close_requested: bool,
    #[allow(dead_code)]
    pub winit_receiver: Receiver<InternalMessage>,
    #[allow(dead_code)]
    pub app_sender: Sender<InternalMessage>,
    pub retgui_options: RetGuiOptions,
    pub event_reducer: WindowEventReducer,
    pub retgui_app: Box<App>,
}

pub(crate) struct RetGuiWinitState {
    retgui_state: RetGuiState,
}

impl ApplicationHandler for RetGuiWinitState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.retgui_state.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.retgui_state.retgui_app.on_resume(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window: Option<crate::elements::Window> =
            WINDOW_MANAGER.with_borrow_mut(|window_manager| window_manager.get_window_by_id(window_id));

        let window = if let Some(window) = window { window } else { return };

        let retgui_state = &mut self.retgui_state;

        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match retgui_state
                .event_reducer
                .reduce(window.effective_scale_factor(), &event)
            {
                Some(WindowEventTranslation::Keyboard(keyboard_event)) => {
                    use ui_events::keyboard::{Key, NamedKey};
                    if keyboard_event.state.is_down() && matches!(keyboard_event.key, Key::Named(NamedKey::Escape)) {
                        event_loop.exit();
                    } else {
                        retgui_state.retgui_app.on_keyboard_input(window, keyboard_event);
                    }
                    return;
                }
                Some(WindowEventTranslation::Pointer(pointer_event)) => {
                    match pointer_event {
                        PointerEvent::Down(pointer_button_update) => {
                            retgui_state
                                .retgui_app
                                .on_pointer_button(window, pointer_button_update, false);
                        }
                        PointerEvent::Up(pointer_button_update) => {
                            retgui_state
                                .retgui_app
                                .on_pointer_button(window, pointer_button_update, true);
                        }
                        PointerEvent::Move(pointer_update) => {
                            retgui_state.retgui_app.on_pointer_moved(window, pointer_update);
                        }
                        PointerEvent::Cancel(_) => {}
                        PointerEvent::Enter(_) => {}
                        PointerEvent::Leave(_) => {}
                        PointerEvent::Scroll(pointer_scroll_update) => {
                            retgui_state.retgui_app.on_pointer_scroll(window, pointer_scroll_update);
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
                self.on_close_requested(&window);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                retgui_state.retgui_app.on_scale_factor_changed(window, scale_factor);
            }
            WindowEvent::Resized(new_size) => {
                let new_size = Size::<f32> {
                    width: new_size.width as f32,
                    height: new_size.height as f32,
                };
                retgui_state.retgui_app.on_resize(window, new_size);
            }
            WindowEvent::Ime(ime) => {
                retgui_state.retgui_app.on_ime(window, ime);
            }
            WindowEvent::RedrawRequested => {
                retgui_state.retgui_app.on_request_redraw(window);
            }
            WindowEvent::Moved(_) => {
                retgui_state.retgui_app.on_move(window);
            }
            WindowEvent::Focused(focused) => {
                window.on_focused(focused);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.retgui_state.runtime.update_local_set();
        self.process_non_winit_window_events(event_loop);
        self.process_retgui_messages();
        self.process_external_work();
        self.retgui_state.retgui_app.on_about_to_wait(event_loop);
        self.maybe_exit(event_loop);

        let perf_stats_enabled = WINDOW_MANAGER.with_borrow(|window_manager| window_manager.any_perf_stats_enabled());
        if perf_stats_enabled {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.retgui_state.retgui_app.on_suspended(event_loop);
    }
}

impl RetGuiWinitState {
    pub(crate) fn new(retgui_state: RetGuiState) -> Self {
        Self { retgui_state }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn process_retgui_messages(&mut self) {
        self.retgui_state.runtime.borrow_tokio_runtime().block_on(async {
            while let Ok(message) = self.retgui_state.winit_receiver.try_recv() {
                match message {
                    InternalMessage::ResourceEvent(resource_event) => {
                        self.retgui_state.retgui_app.on_resource_event(resource_event);
                    }
                    #[cfg(target_arch = "wasm32")]
                    InternalMessage::RendererCreated(window, renderer) => {}
                }
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn process_retgui_messages(&mut self) {
        WASM_QUEUE.with_borrow_mut(|wasm_queue: &mut WasmQueue| {
            wasm_queue.drain(|message| match message {
                InternalMessage::ResourceEvent(resource_event) => {
                    self.retgui_state.retgui_app.on_resource_event(resource_event);
                }
                #[cfg(target_arch = "wasm32")]
                InternalMessage::RendererCreated(winit_window, renderer) => {
                    WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                        let window = window_manager.get_window_by_id(winit_window.id()).unwrap();
                        let sz = Size::new(
                            winit_window.inner_size().width as f32,
                            winit_window.inner_size().height as f32,
                        );
                        window.inner.borrow_mut().on_renderer_created(renderer, sz);
                    });
                }
            });
        });
    }

    fn process_external_work(&mut self) {
        // Elements changed by scheduled work request their own redraws.
        let mut timer_jobs: Vec<Job> = vec![];
        while let Some(mut work) = pop_gui_thread_work() {
            if work.interval.is_none() || work.last_run.elapsed() >= work.interval.unwrap() {
                (work.callback)();
                work.last_run = Instant::now();
            }

            if work.interval.is_some() {
                timer_jobs.push(work);
            }
        }

        for timer_job in timer_jobs {
            push_gui_thread_work(timer_job);
        }
    }

    fn maybe_exit(&mut self, event_loop: &ActiveEventLoop) {
        if self.retgui_state.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
        }
    }

    fn process_non_winit_window_events(&mut self, event_loop: &ActiveEventLoop) {
        while let Some((window_id, event)) = dequeue_window_event() {
            self.window_event(event_loop, window_id, event);
        }
    }

    fn on_close_requested(&mut self, window: &Window) {
        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.close_window(window);
            if window_manager.is_empty() {
                self.retgui_state.close_requested = true;
                self.retgui_state.retgui_app.on_close_requested();
            }
        });
    }
}

impl RetGuiState {
    pub(crate) fn new(
        runtime: RetGuiRuntimeHandle,
        winit_receiver: Receiver<InternalMessage>,
        app_sender: Sender<InternalMessage>,
        retgui_options: RetGuiOptions,
        retgui_app: Box<App>,
    ) -> Self {
        Self {
            runtime,
            wait_cancelled: false,
            close_requested: false,
            winit_receiver,
            app_sender,
            retgui_options,
            event_reducer: Default::default(),
            retgui_app,
        }
    }
}
