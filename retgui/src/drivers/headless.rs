use retgui_primitives::geometry::{Point, Size};

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, Ime, MouseButton, WindowEvent};

use crate::app::{App, WINDOW_MANAGER, WindowEventResult};
use crate::drivers::Driver;
use crate::elements::{Element, Window};

const MAX_SETTLE_PASSES: usize = 64;

pub struct HeadlessEvent {
    window: Window,
    winit_event: WindowEvent,
}

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

        self.app.on_about_to_wait(None);
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
    fn run(&mut self) {
        self.tick();
    }
}

pub struct HeadlessApp {
    driver: HeadlessDriver,
    windows: Vec<Window>,
}

pub fn run<F>(name: &str, test: F)
where
    F: FnOnce(&mut HeadlessApp),
{
    let app = crate::create_app(crate::RetGuiOptions::basic(name));
    HeadlessApp::run_with_app(app, test);
}

impl HeadlessApp {
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
        if !self
            .windows
            .iter()
            .any(|registered| std::rc::Rc::ptr_eq(&registered.inner, &window.inner))
        {
            self.windows.push(window.clone());
        }

        self.driver.app.on_resize(window.clone(), size);
        self.drive();
    }

    pub fn resize(&mut self, window: &Window, width: u32, height: u32) {
        self.driver
            .send_event(window.clone(), WindowEvent::Resized(PhysicalSize::new(width, height)));
        self.drive();
    }

    pub fn click<E: Element>(&mut self, element: &E) {
        let bounds = element.borrow().get_computed_box_transformed().padding_rectangle();
        let point = Point::new(
            (bounds.x + bounds.width / 2.0) as f64,
            (bounds.y + bounds.height / 2.0) as f64,
        );
        let window = Self::element_window(element);

        self.enqueue_pointer_move(&window, point);
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
            .send_event(window.clone(), WindowEvent::Ime(Ime::Commit(text.into())));
        self.drive();
    }

    pub fn keyboard_input(&mut self, window: &Window, event: ui_events::keyboard::KeyboardEvent) {
        self.driver.app.on_keyboard_input(window.clone(), event);
        self.drive();
    }

    pub fn frame(&mut self, window: &Window) {
        self.driver.send_event(window.clone(), WindowEvent::RedrawRequested);
        self.driver.tick();
    }

    pub fn drive(&mut self) {
        let mut idle_passes = 0;
        for _ in 0..MAX_SETTLE_PASSES {
            let events_processed = self.driver.tick();
            let dirty_windows: Vec<Window> = self
                .windows
                .iter()
                .filter(|window| window.redraw_requested())
                .cloned()
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

    pub fn screenshot(&self, window: &Window) -> retgui_renderer::renderer::Screenshot {
        window.screenshot()
    }

    fn enqueue_pointer_move(&self, window: &Window, point: Point) {
        let scale_factor = window.effective_scale_factor();
        self.driver.send_event(
            window.clone(),
            WindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(point.x * scale_factor, point.y * scale_factor),
            },
        );
    }

    fn enqueue_pointer_button(&self, window: &Window, state: ElementState) {
        self.driver.send_event(
            window.clone(),
            WindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state,
                button: MouseButton::Left,
            },
        );
    }

    fn element_window<E: Element>(element: &E) -> Window {
        let window = element
            .borrow()
            .element_data()
            .window
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .expect("element must be attached to a window before interaction");
        Window { inner: window }
    }
}

impl Drop for HeadlessApp {
    fn drop(&mut self) {
        WINDOW_MANAGER.with_borrow_mut(|window_manager| window_manager.clear());
    }
}

#[cfg(test)]
mod harness_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::elements::{Container, Element, Window};
    use crate::{RendererType, px};
    use retgui_primitives::geometry::Size;

    use super::run;

    #[test]
    fn headless_click_dispatches_to_the_target_window() {
        run("headless click", |test| {
            let clicked = Rc::new(Cell::new(false));
            let clicked_copy = clicked.clone();
            let button = Container::new()
                .width(px(100))
                .height(px(50))
                .on_pointer_button_up(move |_| clicked_copy.set(true));
            let window = Window::new_with_renderer("Headless", RendererType::Blank)
                .width(px(200))
                .height(px(100))
                .push(button.clone());

            test.open(&window, Size::new(200.0, 100.0));
            test.click(&button);

            assert!(clicked.get());
        });
    }

    #[test]
    fn headless_click_is_routed_to_the_associated_window() {
        run("headless multi-window click", |test| {
            let first_clicked = Rc::new(Cell::new(false));
            let first_clicked_copy = first_clicked.clone();
            let first_button = Container::new()
                .width(px(100))
                .height(px(50))
                .on_pointer_button_up(move |_| first_clicked_copy.set(true));
            let first_window = Window::new_with_renderer("First", RendererType::Blank)
                .width(px(200))
                .height(px(100))
                .push(first_button);

            let second_clicked = Rc::new(Cell::new(false));
            let second_clicked_copy = second_clicked.clone();
            let second_button = Container::new()
                .width(px(100))
                .height(px(50))
                .on_pointer_button_up(move |_| second_clicked_copy.set(true));
            let second_window = Window::new_with_renderer("Second", RendererType::Blank)
                .width(px(200))
                .height(px(100))
                .push(second_button.clone());

            test.open(&first_window, Size::new(200.0, 100.0));
            test.open(&second_window, Size::new(200.0, 100.0));
            test.click(&second_button);

            assert!(!first_clicked.get());
            assert!(second_clicked.get());
        });
    }

    #[test]
    fn drawing_clears_the_window_redraw_flag() {
        run("headless redraw flag", |test| {
            let window = Window::new_with_renderer("Redraw", RendererType::Blank)
                .width(px(200))
                .height(px(100));

            test.open(&window, Size::new(200.0, 100.0));
            assert!(!window.redraw_requested());

            window.request_redraw();
            assert!(window.redraw_requested());

            test.drive();
            assert!(!window.redraw_requested());
        });
    }

    #[cfg(feature = "vello_cpu_renderer")]
    #[test]
    fn headless_cpu_renderer_produces_a_screenshot() {
        run("headless screenshot", |test| {
            let window = Window::new_with_renderer("Screenshot", RendererType::VelloCPU)
                .width(px(80))
                .height(px(40))
                .push(Container::new().width(px(80)).height(px(40)));

            test.open(&window, Size::new(80.0, 40.0));
            let screenshot = test.screenshot(&window);

            assert_eq!(screenshot.width, 80);
            assert_eq!(screenshot.height, 40);
            assert_eq!(screenshot.pixels.len(), 80 * 40 * 4);
        });
    }
}
