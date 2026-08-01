//! Stores one or more elements.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use craft_logging::info;

use craft_primitives::geometry::{Affine, Point, Rectangle, Size};

use craft_renderer::RendererType;
use craft_renderer::renderer::{Renderer, Screenshot};

use craft_resource_manager::ResourceManager;

use peniko::Color;

use taffy::{AvailableSpace, NodeId};

use ui_events::ScrollDelta;
use ui_events::ScrollDelta::PixelDelta;
use ui_events::keyboard::{KeyboardEvent, Modifiers, NamedKey};
use ui_events::pointer::PointerScrollEvent;

use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowAttributes};

use crate::accessibility::CraftAccessTree;
use crate::app::{App, TAFFY_TREE, WINDOW_MANAGER, queue_window_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
#[cfg(target_arch = "wasm32")]
use crate::events::internal::InternalMessage;
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::Overflow;
use crate::text::text_context::TextContext;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;
use craft_renderer::blank_renderer::BlankRenderer;
#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, winit::platform::web::WindowAttributesExtWebSys};

pub type WindowConstructor = Box<dyn FnMut(&ActiveEventLoop) -> WinitWindow>;

#[derive(Clone)]
pub struct Window {
    pub inner: Rc<RefCell<WindowInternal>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct WindowInternal {
    /// The physical window size from winit.
    pub(crate) window_size: Size<f32>,
    pub(crate) renderer: Rc<RefCell<dyn Renderer>>,

    // Will be empty when paused.
    pub(crate) winit_window: Option<Arc<WinitWindow>>,

    pub(crate) access_tree: CraftAccessTree,
    pub(crate) pointer_capture: Rc<RefCell<PointerCapture>>,

    advanced_window_fn: Option<WindowConstructor>,
    title: Option<String>,
    /// The type of renderer to use.
    ///
    /// The renderer is chosen based on the features enabled at compile time.
    /// See [`RendererType`] for details.
    renderer_type: RendererType,
    /// The window's scale factor from winit.
    scale_factor: f64,
    /// Zoom scale factor.
    zoom_scale_factor: f64,
    mouse_positon: Option<Point>,
    element_data: ElementData,
    pub(crate) modifiers: Modifiers,
}

impl Clone for WindowInternal {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new("Craft")
    }
}

impl Element for Window {}

impl Drop for WindowInternal {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Window {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl crate::elements::ElementData for WindowInternal {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for WindowInternal {
    fn pointer_capture(&self) -> Option<Rc<RefCell<PointerCapture>>> {
        Some(self.pointer_capture.clone())
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, message: &EventKind, _text_context: &mut TextContext, event: &mut Event) {
        scrollable::handle_scroll_logic(self, message, event);
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.apply_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        todo!()
    }
}

impl Window {
    pub fn new_advanced<F>(window_fn: F, renderer_type: RendererType) -> Self
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        let inner = WindowInternal::new(Some(window_fn), None, renderer_type);

        Window { inner }
    }

    pub fn new(title: &str) -> Self {
        let inner = WindowInternal::new(
            None::<fn(&ActiveEventLoop) -> WinitWindow>,
            Some(title),
            RendererType::default(),
        );

        Window { inner }
    }

    pub fn new_with_renderer(title: &str, renderer_type: RendererType) -> Self {
        let inner = WindowInternal::new(None::<fn(&ActiveEventLoop) -> WinitWindow>, Some(title), renderer_type);

        Window { inner }
    }

    pub fn screenshot(&self) -> Screenshot {
        self.inner.borrow().screenshot()
    }

    pub fn close(&self) {
        self.inner.borrow().close();
    }

    pub fn winit_window(&self) -> Option<Arc<winit::window::Window>> {
        self.inner.borrow().winit_window()
    }

    pub fn set_winit_window(&self, window: Option<Arc<WinitWindow>>) {
        self.inner.borrow_mut().set_winit_window(window)
    }

    pub fn set_scale_factor(&self, scale_factor: f64) {
        self.inner.borrow_mut().set_scale_factor(scale_factor)
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self) -> f64 {
        self.inner.borrow().effective_scale_factor()
    }

    /// Get the logical size of the window.
    pub fn window_size(&self) -> Size<f32> {
        self.inner.borrow().window_size()
    }

    pub fn zoom_scale_factor(&self) -> f64 {
        self.inner.borrow().zoom_scale_factor()
    }

    pub fn on_request_redraw(&self, craft_app: &mut App) {
        self.inner.borrow_mut().on_request_redraw(craft_app)
    }

    pub fn zoom_in(&self) {
        self.inner.borrow_mut().zoom_in()
    }

    pub fn zoom_out(&self) {
        self.inner.borrow_mut().zoom_out()
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.inner.borrow().mouse_position()
    }

    pub(crate) fn on_resize(&self, new_size: Size<f32>) {
        self.inner.borrow_mut().on_resize(new_size)
    }

    pub(crate) fn set_mouse_position(&self, point: Option<Point>) {
        self.inner.borrow_mut().set_mouse_position(point)
    }

    pub(crate) fn on_redraw(&self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) {
        self.inner.borrow_mut().on_redraw(text_context, resource_manager)
    }

    pub(crate) fn create(&self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        self.inner.borrow_mut().create(craft_app, event_loop)
    }

    pub(crate) fn on_scale_factor_changed(&self, scale_factor: f64) {
        self.inner.borrow_mut().on_scale_factor_changed(scale_factor);
    }

    pub(crate) fn on_focused(&self, focused: bool) {
        self.inner.borrow().on_focused(focused);
    }
}

impl WindowInternal {
    pub fn new<F>(f: Option<F>, title: Option<&str>, renderer_type: RendererType) -> Rc<RefCell<Self>>
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        let access_tree = crate::accessibility::access_tree();
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), true),
                window_size: Default::default(),
                scale_factor: 1.0,
                zoom_scale_factor: 1.0,
                mouse_positon: None,
                renderer: Rc::new(RefCell::new(BlankRenderer::default())),
                winit_window: None,
                access_tree: access_tree.clone(),
                advanced_window_fn: f.map(|f| Box::new(f) as WindowConstructor),
                title: title.map(|title| title.to_string()),
                renderer_type,
                pointer_capture: Default::default(),
                modifiers: Default::default(),
            })
        });

        inner.borrow_mut().element_data.create_layout_node(None);

        let me = Rc::downgrade(&inner);
        inner.borrow_mut().element_data.window = Some(me);

        {
            let mut inner_mut = inner.borrow_mut();
            inner_mut.element_data.set_accessibility_role(issho::Role::Window);
            if let Some(title) = inner_mut.title.clone() {
                inner_mut.element_data.set_accessibility_name(title);
            }
        }

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.add_window(Window {
                inner: inner.clone(),
            });
        });

        inner
    }

    pub fn request_redraw(&self) {
        if let Some(winit_window) = &self.winit_window {
            winit_window.request_redraw();
        }
    }

    pub(crate) fn on_focused(&self, focused: bool) {
        {
            let root = self
                .element_data
                .access_root
                .expect("window accessibility root is not attached");
            if focused {
                let focus = crate::app::FOCUS.with(|focus| {
                    focus.borrow().as_ref().and_then(Weak::upgrade).and_then(|element| {
                        let element = element.borrow();
                        let data = element.element_data();
                        let belongs_to_window =
                            data.access_tree.ptr_eq(&self.access_tree) && data.access_root == Some(root);
                        belongs_to_window.then_some(data.access_key).flatten()
                    })
                });
                self.access_tree.set_focus(root, Some(focus.unwrap_or(root)));
            } else {
                self.access_tree.set_focus(root, None);
            }
        }
    }

    pub fn winit_window(&self) -> Option<Arc<winit::window::Window>> {
        self.winit_window.clone()
    }

    pub fn set_winit_window(&mut self, window: Option<Arc<WinitWindow>>) {
        self.winit_window = window;
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self) -> f64 {
        self.scale_factor * self.zoom_scale_factor
    }

    /// Get the logical size of the window.
    pub fn window_size(&self) -> Size<f32> {
        Size::new(
            self.window_size.width / self.effective_scale_factor() as f32,
            self.window_size.height / self.effective_scale_factor() as f32,
        )
    }

    pub fn update_zoom(&mut self) {
        let scale_factor = self.effective_scale_factor();
        self.set_scale_factor(scale_factor);
        self.mark_dirty();
        self.request_redraw();
    }

    pub fn on_request_redraw(&mut self, craft_app: &mut App) {
        self.on_redraw(
            craft_app.text_context.as_mut().unwrap(),
            craft_app.resource_manager.clone(),
        );
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_scale_factor += 0.01;
        self.update_zoom();
    }

    pub(crate) fn zoom_out(&mut self) {
        self.zoom_scale_factor = (self.zoom_scale_factor - 0.01).max(1.0);
        self.update_zoom();
    }

    pub(crate) fn maybe_zoom(&mut self, pointer_scroll_update: &PointerScrollEvent) -> bool {
        if self.modifiers.ctrl() && pointer_scroll_update.pointer.pointer_type == ui_events::pointer::PointerType::Mouse
        {
            let y: f32 = match pointer_scroll_update.delta {
                ScrollDelta::PageDelta(_, y) => y,
                ScrollDelta::LineDelta(_, y) => y,
                PixelDelta(physical) => physical.y as f32,
            };
            if y < 0.0 {
                self.zoom_out();
            } else {
                self.zoom_in();
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn maybe_zoom_keyboard(&mut self, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.modifiers.ctrl() {
            if keyboard_input.key == ui_events::keyboard::Key::Character("=".to_string()) {
                self.zoom_in();
                return true;
            } else if keyboard_input.key == ui_events::keyboard::Key::Character("-".to_string()) {
                self.zoom_out();
                return true;
            }
        }
        false
    }

    pub(crate) fn update_modifiers(&mut self, keyboard_input: &KeyboardEvent) {
        self.modifiers = keyboard_input.modifiers;
        if keyboard_input.key == ui_events::keyboard::Key::Named(NamedKey::Control) && keyboard_input.state.is_up() {
            self.modifiers.set(Modifiers::CONTROL, false);
        }
    }

    pub(crate) fn zoom_scale_factor(&self) -> f64 {
        self.zoom_scale_factor
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.mouse_positon
    }

    pub(crate) fn on_resize(&mut self, new_size: Size<f32>) {
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            taffy_tree.mark_dirty(self.element_data.layout.taffy_node_id.unwrap());
        });

        self.window_size = new_size;
        let size = self.window_size;

        self.renderer
            .borrow_mut()
            .resize_surface(new_size.width.max(1.0), new_size.height.max(1.0));
        self.renderer
            .borrow_mut()
            .set_cull(Some(Rectangle::new(0.0, 0.0, size.width, size.height)));

        // On macOS the window needs to be redrawn manually after resizing
        #[cfg(target_os = "macos")]
        {
            // TODO: Fix
            //self.window.as_ref().unwrap().request_redraw();
        }
    }

    pub(crate) fn set_mouse_position(&mut self, point: Option<Point>) {
        self.mouse_positon = point;
    }

    pub(crate) fn on_redraw(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) {
        //if self.renderer.is_none() {
        //    return;
        //}

        self.renderer.borrow_mut().surface_set_clear_color(Color::WHITE);

        self.layout_window(text_context, resource_manager.clone());

        self.draw_window(text_context, resource_manager);
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.set_scale_factor(self.effective_scale_factor());
        self.on_resize(self.window_size);
    }

    pub(crate) fn create(&mut self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        let winit_window: Arc<WinitWindow> = Arc::new(if let Some(window_fn) = &mut self.advanced_window_fn {
            (*window_fn)(event_loop)
        } else {
            let window_attributes = WindowAttributes::default()
                .with_title(self.title.as_ref().unwrap())
                .with_visible(false);
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

            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window")
        });
        self.set_winit_window(Some(winit_window.clone()));
        self.on_scale_factor_changed(winit_window.scale_factor());

        let renderer_type = self.renderer_type;

        cfg_select! {
            not(target_arch = "wasm32") => {
                    let renderer = craft_app.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Rc<RefCell<dyn Renderer>> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                self.renderer = renderer;
                info!("Created renderer")
            },
            _ => {
                let window_copy_2 = winit_window.clone();
                craft_app.runtime.spawn(async move {
                    let renderer: Rc<RefCell<dyn Renderer>> = renderer_type.create(window_copy_2.clone()).await;
                    WASM_QUEUE.with_borrow_mut(|wasm_queue| {
                        wasm_queue.push(InternalMessage::RendererCreated(window_copy_2.clone(), renderer));
                    });
                    info!("Created renderer")
                });
            }
        }

        {
            self.on_request_redraw(craft_app);
            let root = self
                .element_data
                .access_root
                .expect("window accessibility root is not attached");
            self.access_tree.set_root_window(root, winit_window.clone());
            self.access_tree.set_focus(root, Some(root));
            self.access_tree.register_window(winit_window.clone());
        }

        winit_window.set_visible(true);
    }

    fn layout_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) -> NodeId {
        let root_node = self
            .element_data
            .layout
            .taffy_node_id
            .expect("A root element must have a layout node.");

        let window_size = self.window_size();
        let available_space: taffy::Size<AvailableSpace> = taffy::Size {
            width: AvailableSpace::Definite(window_size.width),
            height: AvailableSpace::Definite(window_size.height),
        };

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let root_dirty = taffy_tree.is_layout_dirty(root_node);

            if root_dirty {
                /*let span = span!(Level::INFO, "layout(taffy)");
                let _enter = span.enter();*/
                taffy_tree.compute_layout(root_node, available_space, text_context, resource_manager.clone());
            }

            //if self.taffy_tree.borrow().is_apply_layout_dirty() {
            /*let span = span!(Level::INFO, "layout(apply)");
            let _enter = span.enter();*/

            if root_dirty || taffy_tree.is_apply_layout_dirty(&root_node) {
                // TODO: move into taffy_tree
                let mut layout_order: u32 = 0;
                let sf = self.effective_scale_factor();
                self.apply_layout(
                    taffy_tree,
                    Point::new(0.0, 0.0),
                    &mut layout_order,
                    Affine::IDENTITY,
                    text_context,
                    Some(Rectangle::new(
                        0.0,
                        0.0,
                        self.window_size.width,
                        self.window_size.height,
                    )),
                    sf,
                );
                taffy_tree.apply_layout(root_node);
            }
            //}
        });

        root_node
    }

    fn draw_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) {
        let renderer_clone = self.renderer.clone();
        self.renderer.borrow_mut().clear();

        self.draw(
            &mut *renderer_clone.borrow_mut(),
            resource_manager.clone(),
            self.effective_scale_factor(),
            text_context,
        );

        self.winit_window.clone().unwrap().pre_present_notify();

        {
            let renderer = renderer_clone.clone();
            renderer.borrow_mut().sort_render_list();

            let window = Rectangle::new(
                0.0,
                0.0,
                renderer.borrow().surface_width(),
                renderer.borrow().surface_height(),
            );
            renderer.borrow_mut().prepare(resource_manager.clone(), window);
            renderer.borrow_mut().submit(resource_manager.clone());
        }
    }

    fn screenshot(&self) -> Screenshot {
        self.renderer.borrow_mut().screenshot()
    }

    fn close(&self) {
        if let Some(winit_window) = &self.winit_window {
            queue_window_event(winit_window.id(), WindowEvent::CloseRequested);
        }
    }
}
