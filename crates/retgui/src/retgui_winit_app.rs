//! Integration with the winit event loop.

use retgui_logging::info;
#[cfg(not(target_arch = "wasm32"))]
use std::time;

#[cfg(target_arch = "wasm32")]
use web_time as time;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopBuilder};
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::WindowId;

use crate::app::{App, WINDOW_MANAGER, WindowEventResult};
use crate::driver::Driver;

const WAIT_TIME: time::Duration = time::Duration::from_millis(10);

pub(crate) struct RetGuiWinitApp {
    app: App,
}

impl ApplicationHandler for RetGuiWinitApp {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.app.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app.on_resume(Some(event_loop));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window: Option<crate::elements::Window> =
            WINDOW_MANAGER.with_borrow_mut(|window_manager| window_manager.get_window_by_id(window_id));

        let window = if let Some(window) = window { window } else { return };

        match self.app.on_window_event(window, event) {
            WindowEventResult::Continue => {}
            WindowEventResult::ExitRequested => event_loop.exit(),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.app.on_about_to_wait(Some(event_loop));
        self.maybe_exit(event_loop);

        let perf_stats_enabled = WINDOW_MANAGER.with_borrow(|window_manager| window_manager.any_perf_stats_enabled());
        if perf_stats_enabled {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.app.on_suspended(event_loop);
    }
}

impl Driver for RetGuiWinitApp {
    fn run(&mut self) {
        let mut event_loop_builder = EventLoopBuilder::default();

        #[cfg(target_os = "android")]
        {
            let app = crate::ANDROID_APP
                .take()
                .expect("retgui_set_android_app must be called.");
            event_loop_builder.with_android_app(app);
        }

        let event_loop = event_loop_builder.build().expect("Failed to create winit event loop.");
        info!("Created winit event loop.");
        event_loop.run_app(self).expect("run_app failed");
    }
}

impl RetGuiWinitApp {
    pub(crate) fn new(app: App) -> Self {
        Self { app }
    }

    fn maybe_exit(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.close_requested {
            info!("Exiting winit event loop");

            event_loop.exit();
        }
    }
}
