use retgui_primitives::geometry::{Point, Size};

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ButtonSource, ElementState, Ime, KeyEvent, MouseButton, PointerSource, WindowEvent};

use crate::app::{App, WindowEventResult};
use crate::drivers::Driver;
use crate::elements::{Element, Window, WindowElement};

const MAX_SETTLE_PASSES: usize = 64;

pub struct HeadlessEvent {
    window: Window,
    winit_event: WindowEvent,
}

/// A driver without an OS backed window.
pub struct HeadlessDriver {
    event_receiver: std::sync::mpsc::Receiver<HeadlessEvent>,
    event_sender: std::sync::mpsc::Sender<HeadlessEvent>,
    app: App,
}

impl HeadlessDriver {
    pub fn new(app: App) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            event_receiver: rx,
            event_sender: tx,
            app,
        }
    }

    pub(crate) fn send_event(&self, window: Window, winit_event: WindowEvent) {
        self.event_sender
            .send(HeadlessEvent {
                window,
                winit_event,
            })
            .expect("test event receiver unexpectedly disconnected");
    }

    fn tick(&mut self) -> usize {
        let mut events_processed = 0;
        while let Ok(event) = self.event_receiver.try_recv() {
            events_processed += 1;
            self.dispatch_event(event);
        }

        let _ = self.app.on_about_to_wait(None);
        events_processed
    }

    pub fn dispatch_event(&mut self, event: HeadlessEvent) {
        match self.app.on_window_event(event.window, event.winit_event) {
            WindowEventResult::Continue => {}
            WindowEventResult::ExitRequested => {
                self.app.close_requested = true;
            }
        }
    }
}

impl Driver for HeadlessDriver {
    fn new(app: App) -> Self {
        Self::new(app)
    }

    fn run(mut self) {
        self.app.on_resume(None);
        self.tick();
    }
}

pub struct HeadlessApp {
    driver: HeadlessDriver,
    windows: Vec<Window>,
}

pub fn run<T, F>(_name: &str, build: impl FnOnce(&mut App) -> T, test: F)
where
    F: FnOnce(&mut HeadlessApp, T),
{
    let mut app = App::new();
    let test_state = build(&mut app);
    HeadlessApp::run_with_app(app, |app| test(app, test_state));
}

impl HeadlessApp {
    pub fn app(&self) -> &App {
        &self.driver.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.driver.app
    }

    pub(crate) fn new(app: App) -> Self {
        let mut driver = HeadlessDriver::new(app);
        driver.app.on_resume(None);
        Self {
            driver,
            windows: Vec::new(),
        }
    }

    fn run_with_app<F>(app: App, test: F)
    where
        F: FnOnce(&mut Self),
    {
        let mut headless_app = Self::new(app);
        test(&mut headless_app);
        headless_app.drive();
    }

    pub fn open(&mut self, window: &Window, size: Size<f32>) {
        if !self.windows.iter().any(|registered| registered.inner == window.inner) {
            self.windows.push(*window);
        }

        self.driver.app.on_resize(*window, size);
        self.drive();
    }

    pub fn resize(&mut self, window: &Window, width: u32, height: u32) {
        self.driver
            .send_event(*window, WindowEvent::SurfaceResized(PhysicalSize::new(width, height)));
        self.drive();
    }

    pub fn click<E: Element>(&mut self, element: &E) {
        let bounds = self
            .driver
            .app
            .elements
            .get(element.as_dyn_element())
            .get_computed_box_transformed()
            .padding_rectangle();
        let point = Point::new(
            (bounds.x + bounds.width / 2.0) as f64,
            (bounds.y + bounds.height / 2.0) as f64,
        );
        let window = self.element_window(element);

        self.enqueue_pointer_move(&window, point);
        self.drive();
        self.enqueue_pointer_button(&window, ElementState::Pressed);
        self.enqueue_pointer_button(&window, ElementState::Released);
        self.drive();
    }

    pub fn pointer_move(&mut self, window: &Window, point: Point) {
        self.enqueue_pointer_move(window, point);
        self.drive();
    }

    pub fn pointer_down(&mut self, window: &Window) {
        self.enqueue_pointer_button(window, ElementState::Pressed);
        self.drive();
    }

    pub fn pointer_up(&mut self, window: &Window) {
        self.enqueue_pointer_button(window, ElementState::Released);
        self.drive();
    }

    pub fn type_text(&mut self, window: &Window, text: impl Into<String>) {
        self.driver
            .send_event(*window, WindowEvent::Ime(Ime::Commit(text.into())));
        self.drive();
    }

    pub fn ime(&mut self, window: &Window, event: Ime) {
        self.driver.send_event(*window, WindowEvent::Ime(event));
        self.drive();
    }

    pub fn keyboard_input(&mut self, window: &Window, event: KeyEvent) {
        self.driver.app.on_keyboard_input(*window, event);
        self.drive();
    }

    pub fn frame(&mut self, window: &Window) {
        self.driver.send_event(*window, WindowEvent::RedrawRequested);
        self.driver.tick();
    }

    pub fn close(&mut self, window: &Window) {
        self.driver.send_event(*window, WindowEvent::CloseRequested);
        self.drive();
    }

    pub fn drive(&mut self) {
        let mut idle_passes = 0;
        for _ in 0..MAX_SETTLE_PASSES {
            let events_processed = self.driver.tick();
            let dirty_windows: Vec<Window> = self
                .windows
                .iter()
                .filter(|window| {
                    self.driver
                        .app
                        .elements
                        .get_as::<WindowElement>(window.inner)
                        .redraw_requested()
                })
                .copied()
                .collect();

            if dirty_windows.is_empty() && events_processed == 0 {
                idle_passes += 1;
                if idle_passes == 2 {
                    return;
                }
            } else {
                idle_passes = 0;
            }

            for window in dirty_windows {
                self.driver.send_event(window, WindowEvent::RedrawRequested);
            }
        }

        panic!("RetGui test application did not settle after {MAX_SETTLE_PASSES} passes");
    }

    pub fn screenshot(&mut self, window: &Window) -> retgui_renderer::renderer::Screenshot {
        window.screenshot(&mut self.driver.app)
    }

    fn enqueue_pointer_move(&self, window: &Window, point: Point) {
        let scale_factor = self
            .driver
            .app
            .elements
            .get_as::<WindowElement>(window.inner)
            .effective_scale_factor();
        self.driver.send_event(
            *window,
            WindowEvent::PointerMoved {
                device_id: None,
                position: PhysicalPosition::new(point.x * scale_factor, point.y * scale_factor),
                primary: true,
                source: PointerSource::Mouse,
            },
        );
    }

    fn enqueue_pointer_button(&self, window: &Window, state: ElementState) {
        let window_element = self.driver.app.elements.get_as::<WindowElement>(window.inner);
        let position = window_element.mouse_position().unwrap_or_default();
        let scale_factor = window_element.effective_scale_factor();
        self.driver.send_event(
            *window,
            WindowEvent::PointerButton {
                device_id: None,
                state,
                position: PhysicalPosition::new(position.x * scale_factor, position.y * scale_factor),
                primary: true,
                button: ButtonSource::Mouse(MouseButton::Left),
                is_macos_activation_click: false,
            },
        );
    }

    fn element_window<E: Element>(&self, element: &E) -> Window {
        let window = self
            .driver
            .app
            .elements
            .get(element.as_dyn_element())
            .element_data()
            .window
            .expect("element must be attached to a window before interaction");
        Window { inner: window }
    }
}
