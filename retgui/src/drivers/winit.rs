//! Integration with the winit event loop.

use retgui_logging::info;
#[cfg(not(target_arch = "wasm32"))]
use std::time;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopBuilder};
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::WindowId;

use crate::app::{App, WindowEventResult};
use crate::drivers::Driver;

#[cfg(not(target_arch = "wasm32"))]
const WAIT_TIME: time::Duration = time::Duration::from_millis(10);

/// A winit driver.
pub struct WinitDriver {
    app: App,
}

impl ApplicationHandler for WinitDriver {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.app.on_resume(Some(event_loop));
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(window) = self.app.window_by_id(window_id) else {
            return;
        };
        match self.app.on_window_event(window, event) {
            WindowEventResult::Continue => {}
            WindowEventResult::ExitRequested => event_loop.exit(),
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.app.on_about_to_wait(Some(event_loop));
        self.maybe_exit(event_loop);

        let perf_stats_enabled = self
            .app
            .elements
            .window_manager
            .any_perf_stats_enabled(&self.app.elements);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if perf_stats_enabled {
                // Web redraws are paced by `requestAnimationFrame`. Do not schedule a
                // competing timer while an animation frame is already pending.
                //#[cfg(target_arch = "wasm32")]
                //event_loop.set_control_flow(ControlFlow::Wait);
                event_loop.set_control_flow(ControlFlow::Poll);
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = perf_stats_enabled;
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn suspended(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.app.on_suspended();
    }
}

impl Driver for WinitDriver {
    fn new(app: App) -> Self {
        Self { app }
    }

    fn run(self) {
        let mut event_loop_builder = EventLoopBuilder::default();

        #[cfg(target_os = "android")]
        {
            let app = crate::ANDROID_APP
                .get()
                .cloned()
                .expect("retgui_set_android_app must be called.");
            event_loop_builder.with_android_app(app);
        }

        let event_loop = event_loop_builder.build().expect("Failed to create winit event loop.");
        let proxy = event_loop.create_proxy();
        self.app.set_gui_waker(move || {
            proxy.wake_up();
        });
        info!("Created winit event loop.");
        event_loop.run_app(self).expect("run_app failed");
    }
}

impl WinitDriver {
    fn maybe_exit(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.app.close_requested() {
            info!("Exiting winit event loop");

            event_loop.exit();
        }
    }
}
