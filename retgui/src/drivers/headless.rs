use retgui_primitives::geometry::{Point, Size};

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ButtonSource, ElementState, Ime, KeyEvent, MouseButton, PointerSource, WindowEvent};

use crate::app::{App, WindowEventResult};
use crate::drivers::Driver;
use crate::elements::{Element, Elements, Window};

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

pub fn run<T, F>(_name: &str, build: impl FnOnce(&mut Elements) -> T, test: F)
where
    F: FnOnce(&mut HeadlessApp, T),
{
    let mut elements = Elements::new();
    let test_state = build(&mut elements);
    let app = App::new(elements);
    HeadlessApp::run_with_app(app, |app| test(app, test_state));
}

impl HeadlessApp {
    pub fn elements(&self) -> &Elements {
        &self.driver.app.elements
    }

    pub fn elements_mut(&mut self) -> &mut Elements {
        &mut self.driver.app.elements
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
            self.windows.push(window.clone());
        }

        self.driver.app.on_resize(window.clone(), size);
        self.drive();
    }

    pub fn resize(&mut self, window: &Window, width: u32, height: u32) {
        self.driver.send_event(
            window.clone(),
            WindowEvent::SurfaceResized(PhysicalSize::new(width, height)),
        );
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
            .send_event(window.clone(), WindowEvent::Ime(Ime::Commit(text.into())));
        self.drive();
    }

    pub fn ime(&mut self, window: &Window, event: Ime) {
        self.driver.send_event(window.clone(), WindowEvent::Ime(event));
        self.drive();
    }

    pub fn keyboard_input(&mut self, window: &Window, event: KeyEvent) {
        self.driver.app.on_keyboard_input(window.clone(), event);
        self.drive();
    }

    pub fn frame(&mut self, window: &Window) {
        self.driver.send_event(window.clone(), WindowEvent::RedrawRequested);
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
                .filter(|window| window.redraw_requested(&self.driver.app.elements))
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

    pub fn screenshot(&mut self, window: &Window) -> retgui_renderer::renderer::Screenshot {
        window.screenshot(&mut self.driver.app.elements)
    }

    fn enqueue_pointer_move(&self, window: &Window, point: Point) {
        let scale_factor = window.effective_scale_factor(&self.driver.app.elements);
        self.driver.send_event(
            window.clone(),
            WindowEvent::PointerMoved {
                device_id: None,
                position: PhysicalPosition::new(point.x * scale_factor, point.y * scale_factor),
                primary: true,
                source: PointerSource::Mouse,
            },
        );
    }

    fn enqueue_pointer_button(&self, window: &Window, state: ElementState) {
        self.driver.send_event(
            window.clone(),
            WindowEvent::PointerButton {
                device_id: None,
                state,
                position: PhysicalPosition::new(
                    window.mouse_position(&self.driver.app.elements).unwrap_or_default().x
                        * window.effective_scale_factor(&self.driver.app.elements),
                    window.mouse_position(&self.driver.app.elements).unwrap_or_default().y
                        * window.effective_scale_factor(&self.driver.app.elements),
                ),
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

#[cfg(test)]
mod harness_tests {
    use std::sync::Arc;

    use crate::elements::{Container, DynElement, Element, ElementData, ElementNode, ElementNodeData, Elements, Window, clone_element};
    use crate::events::EventKind;
    use crate::style::AlignSelf;
    use crate::text::text_context::TextContext;
    use crate::{Renderer, RendererType, ResourceManager, px};
    use retgui_primitives::geometry::Size;

    use super::run;

    #[derive(Clone, Copy)]
    struct TestElement {
        inner: DynElement,
    }

    impl Element for TestElement {
        fn as_dyn_element(&self) -> DynElement {
            self.inner
        }
    }

    #[derive(Clone)]
    struct TestElementNode {
        element_data: ElementData,
        toggled: bool,
    }

    impl ElementNodeData for TestElementNode {
        fn element_data(&self) -> &ElementData {
            &self.element_data
        }

        fn element_data_mut(&mut self) -> &mut ElementData {
            &mut self.element_data
        }
    }

    impl ElementNode for TestElementNode {
        fn deep_clone(&self, elements: &mut Elements) -> DynElement {
            clone_element(self, elements, |_, _| None)
        }

        fn draw(
            &self,
            elements: &Elements,
            renderer: &mut dyn Renderer,
            resource_manager: Arc<ResourceManager>,
            scale_factor: f64,
            text_context: &mut TextContext,
        ) {
            if !self.is_visible() {
                return;
            }
            self.add_hit_testable(renderer, true, scale_factor);
            self.draw_children(elements, renderer, resource_manager, scale_factor, text_context);
        }

        fn on_event(&mut self, _elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
            if matches!(event, EventKind::Click(_)) {
                self.toggled = true;
                self.request_window_redraw();
            }
        }
    }

    fn test_element(elements: &mut Elements) -> TestElement {
        let inner = elements.insert_element(true, |element_data| TestElementNode {
            element_data,
            toggled: false,
        });
        TestElement { inner }
    }

    #[test]
    fn element_node_uses_normal_style_layout() {
        run(
            "element style layout",
            |elements| {
                let custom = test_element(elements)
                    .align_self(elements, AlignSelf::Start)
                    .width(elements, px(37))
                    .height(elements, px(19));
                let window = Window::new_with_renderer(elements, "Custom", RendererType::Blank)
                    .width(elements, px(100))
                    .height(elements, px(100))
                    .push(elements, custom);
                (custom, window)
            },
            |test, (custom, window)| {
                test.open(&window, Size::new(100.0, 100.0));
                let bounds = custom.get_computed_box_transformed(test.elements()).border_rectangle();
                assert_eq!(bounds.width, 37.0);
                assert_eq!(bounds.height, 19.0);
            },
        );
    }

    #[test]
    fn element_node_handles_default_events() {
        run(
            "user element event",
            |elements| {
                let element = test_element(elements).width(elements, px(80)).height(elements, px(40));
                let window = Window::new_with_renderer(elements, "Element event", RendererType::Blank)
                    .width(elements, px(100))
                    .height(elements, px(100))
                    .push(elements, element);
                (element, window)
            },
            |test, (element, window)| {
                test.open(&window, Size::new(100.0, 100.0));
                test.click(&element);
                assert!(test.elements().get_as::<TestElementNode>(element.inner).toggled);
            },
        );
    }

    #[test]
    fn closing_a_secondary_window_releases_it_without_exiting() {
        run(
            "secondary window close",
            |elements| {
                let first = Window::new_with_renderer(elements, "First", RendererType::Blank);
                let second = Window::new_with_renderer(elements, "Second", RendererType::Blank);
                (first, second)
            },
            |test, (first, second)| {
                test.open(&first, Size::new(100.0, 100.0));
                test.open(&second, Size::new(100.0, 100.0));

                let second_root = test.elements().get(second.inner).element_data().access_key.unwrap();
                let access_tree = test.elements().get(second.inner).element_data().access_tree.clone();

                test.close(&second);

                assert_eq!(test.driver.app.elements.window_manager.len(), 1);
                assert!(!test.driver.app.close_requested());
                assert!(!access_tree.contains_node(second_root));
                assert!(test.elements().get(second.inner).element_data().access_key.is_none());

                test.close(&first);
                assert!(test.driver.app.close_requested());
            },
        );
    }

    #[test]
    fn headless_click_dispatches_to_the_target_window() {
        run(
            "headless click",
            |elements| {
                let clicked = elements.insert_state(false);
                let button = Container::new(elements)
                    .width(elements, px(100))
                    .height(elements, px(50))
                    .on_pointer_button_up(elements, move |_event, elements| *elements.state_mut(clicked) = true);
                let window = Window::new_with_renderer(elements, "Headless", RendererType::Blank)
                    .width(elements, px(200))
                    .height(elements, px(100))
                    .push(elements, button);
                (clicked, button, window)
            },
            |test, (clicked, button, window)| {
                test.open(&window, Size::new(200.0, 100.0));
                test.click(&button);
                assert!(*test.elements().state(clicked));
            },
        );
    }

    #[test]
    fn headless_click_is_routed_to_the_associated_window() {
        run(
            "headless multi-window click",
            |elements| {
                let first_clicked = elements.insert_state(false);
                let first_button = Container::new(elements)
                    .width(elements, px(100))
                    .height(elements, px(50))
                    .on_pointer_button_up(elements, move |_event, elements| {
                        *elements.state_mut(first_clicked) = true
                    });
                let first_window = Window::new_with_renderer(elements, "First", RendererType::Blank)
                    .width(elements, px(200))
                    .height(elements, px(100))
                    .push(elements, first_button);

                let second_clicked = elements.insert_state(false);
                let second_button = Container::new(elements)
                    .width(elements, px(100))
                    .height(elements, px(50))
                    .on_pointer_button_up(elements, move |_event, elements| {
                        *elements.state_mut(second_clicked) = true
                    });
                let second_window = Window::new_with_renderer(elements, "Second", RendererType::Blank)
                    .width(elements, px(200))
                    .height(elements, px(100))
                    .push(elements, second_button);
                (
                    first_clicked,
                    second_clicked,
                    second_button,
                    first_window,
                    second_window,
                )
            },
            |test, (first_clicked, second_clicked, second_button, first_window, second_window)| {
                test.open(&first_window, Size::new(200.0, 100.0));
                test.open(&second_window, Size::new(200.0, 100.0));
                test.click(&second_button);
                assert!(!*test.elements().state(first_clicked));
                assert!(*test.elements().state(second_clicked));
            },
        );
    }

    #[test]
    fn drawing_clears_the_window_redraw_flag() {
        run(
            "headless redraw flag",
            |elements| {
                Window::new_with_renderer(elements, "Redraw", RendererType::Blank)
                    .width(elements, px(200))
                    .height(elements, px(100))
            },
            |test, window| {
                test.open(&window, Size::new(200.0, 100.0));
                assert!(!window.redraw_requested(test.elements()));

                window.request_redraw(test.elements());
                assert!(window.redraw_requested(test.elements()));

                test.drive();
                assert!(!window.redraw_requested(test.elements()));
            },
        );
    }

    #[cfg(feature = "vello_cpu_renderer")]
    #[test]
    fn headless_cpu_renderer_produces_a_screenshot() {
        run(
            "headless screenshot",
            |elements| {
                let content = Container::new(elements)
                    .width(elements, px(80))
                    .height(elements, px(40));
                Window::new_with_renderer(elements, "Screenshot", RendererType::VelloCPU)
                    .width(elements, px(80))
                    .height(elements, px(40))
                    .push(elements, content)
            },
            |test, window| {
                test.open(&window, Size::new(80.0, 40.0));
                let screenshot = test.screenshot(&window);

                assert_eq!(screenshot.width, 80);
                assert_eq!(screenshot.height, 40);
                assert_eq!(screenshot.pixels.len(), 80 * 40 * 4);
            },
        );
    }
}
