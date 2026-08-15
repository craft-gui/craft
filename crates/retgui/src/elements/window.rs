//! Stores one or more elements.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time;
use time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time as time;

use retgui_logging::info;

use retgui_primitives::geometry::{Point, Rectangle, Size};

use retgui_renderer::RendererType;
use retgui_renderer::renderer::{Renderer, Screenshot};

use retgui_resource_manager::ResourceManager;

use peniko::Color;

use gummy::AvailableSpace;

use ui_events::ScrollDelta;
use ui_events::ScrollDelta::PixelDelta;
use ui_events::keyboard::{KeyboardEvent, Modifiers, NamedKey};
use ui_events::pointer::PointerScrollEvent;

use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowAttributes};

use crate::accessibility::RetGuiAccessTree;
use crate::app::{App, GUMMY_TREE, WINDOW_MANAGER, queue_window_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::{AsElement, Element, ElementInternals, scrollable};
#[cfg(target_arch = "wasm32")]
use crate::events::internal::InternalMessage;
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::perf_stats::{LayoutStats, PerfStats, RenderStats};
use crate::text::text_context::TextContext;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;
use retgui_renderer::blank_renderer::BlankRenderer;
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

    pub(crate) access_tree: RetGuiAccessTree,
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
    perf_stats: PerfStats,
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
        Self::new("RetGui")
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
    fn request_window_redraw(&self) {
        self.request_redraw();
    }

    fn pointer_capture(&self) -> Option<Rc<RefCell<PointerCapture>>> {
        Some(self.pointer_capture.clone())
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
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

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
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
        self.inner.borrow_mut().set_scale_factor(scale_factor);
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

    pub fn on_request_redraw(&self, retgui_app: &mut App) {
        self.inner.borrow_mut().on_request_redraw(retgui_app)
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

    pub(crate) fn create(&self, retgui_app: &mut App, event_loop: &ActiveEventLoop) {
        self.inner.borrow_mut().create(retgui_app, event_loop)
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
                perf_stats: PerfStats::new(),
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
    }

    pub fn on_request_redraw(&mut self, retgui_app: &mut App) {
        self.on_redraw(
            retgui_app.text_context.borrow_mut().as_mut().unwrap(),
            retgui_app.resource_manager.clone(),
        );
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_scale_factor += 0.01;
        self.update_zoom();
    }

    pub(crate) fn zoom_out(&mut self) {
        let zoom_scale_factor = (self.zoom_scale_factor - 0.01).max(1.0);
        if self.zoom_scale_factor == zoom_scale_factor {
            return;
        }
        self.zoom_scale_factor = zoom_scale_factor;
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

    pub(crate) fn maybe_toggle_perf_stats(&mut self, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.repeat || !keyboard_input.state.is_down() {
            return false;
        }

        if keyboard_input.key != ui_events::keyboard::Key::Named(NamedKey::F3) {
            return false;
        }

        self.perf_stats.toggle(&mut *self.renderer.borrow_mut());
        self.request_redraw();
        true
    }

    pub(crate) fn perf_stats_enabled(&self) -> bool {
        self.perf_stats.is_enabled()
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
        if self.window_size.width == new_size.width && self.window_size.height == new_size.height {
            return;
        }
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            gummy_tree.mark_dirty(self.element_data.layout.gummy_node_id.unwrap());
        });

        self.window_size = new_size;
        self.resize_renderer_surface();
        self.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn on_renderer_created(&mut self, renderer: Rc<RefCell<dyn Renderer>>, new_size: Size<f32>) {
        self.renderer = renderer;
        if self.window_size.width != new_size.width || self.window_size.height != new_size.height {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.mark_dirty(self.element_data.layout.gummy_node_id.unwrap());
            });
            self.window_size = new_size;
        }
        self.resize_renderer_surface();
        self.request_redraw();
    }

    fn resize_renderer_surface(&mut self) {
        let size = self.window_size;
        let mut renderer = self.renderer.borrow_mut();
        renderer.resize_surface(size.width.max(1.0), size.height.max(1.0));
        renderer.set_cull(Some(Rectangle::new(0.0, 0.0, size.width, size.height)));
    }

    pub(crate) fn set_mouse_position(&mut self, point: Option<Point>) {
        self.mouse_positon = point;
    }

    pub(crate) fn on_redraw(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) {
        //if self.renderer.is_none() {
        //    return;
        //}

        let frame_start = Instant::now();
        self.renderer.borrow_mut().surface_set_clear_color(Color::WHITE);

        let layout_stats = self.layout_window(text_context, resource_manager.clone());

        let render_stats = self.draw_window(text_context, resource_manager);
        self.perf_stats
            .update_stats(frame_start.elapsed(), layout_stats, render_stats);
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if self.scale_factor == scale_factor {
            return;
        }
        self.scale_factor = scale_factor;
        self.set_scale_factor(self.effective_scale_factor());
    }

    pub(crate) fn create(&mut self, retgui_app: &mut App, event_loop: &ActiveEventLoop) {
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
                let renderer = retgui_app.runtime.borrow_tokio_runtime().block_on(async {
                    let renderer: Rc<RefCell<dyn Renderer>> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                self.renderer = renderer;
                info!("Created renderer")
            }
            _ => {
                let window_copy_2 = winit_window.clone();
                retgui_app.runtime.spawn(async move {
                    let renderer: Rc<RefCell<dyn Renderer>> = renderer_type.create(window_copy_2.clone()).await;
                    WASM_QUEUE.with_borrow_mut(|wasm_queue| {
                        wasm_queue.push(InternalMessage::RendererCreated(window_copy_2.clone(), renderer));
                    });
                    info!("Created renderer")
                });
            }
        }

        {
            self.on_request_redraw(retgui_app);
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

    fn layout_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) -> LayoutStats {
        let total_start = Instant::now();
        let mut compute = Duration::from_secs(0);
        let mut apply = Duration::from_secs(0);

        let root_node = self
            .element_data
            .layout
            .gummy_node_id
            .expect("A root element must have a layout node.");

        let window_size = self.window_size();
        let available_space: gummy::Size<AvailableSpace> = gummy::Size {
            width: AvailableSpace::Definite(window_size.width),
            height: AvailableSpace::Definite(window_size.height),
        };

        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let root_dirty = gummy_tree.is_layout_dirty(root_node);

            if root_dirty {
                let compute_start = Instant::now();
                gummy_tree.compute_layout(root_node, available_space, text_context, resource_manager.clone());
                compute = compute_start.elapsed();
            }

            if root_dirty || gummy_tree.is_apply_layout_dirty(&root_node) {
                let apply_start = Instant::now();
                let sf = self.effective_scale_factor();
                let owners = gummy_tree.take_layout_owners(root_node, root_dirty);
                for (owner_id, owner, layout_order) in owners {
                    let mut layout_order = layout_order;
                    if owner_id == self.element_data.internal_id {
                        self.apply_layout(gummy_tree, &mut layout_order, text_context, sf);
                    } else {
                        owner
                            .borrow_mut()
                            .apply_layout(gummy_tree, &mut layout_order, text_context, sf);
                    }
                }
                gummy_tree.apply_layout(root_node);
                apply = apply_start.elapsed();
            }
            //}
        });

        LayoutStats::new(total_start.elapsed(), compute, apply)
    }

    fn draw_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) -> RenderStats {
        let total_start = Instant::now();
        let renderer_clone = self.renderer.clone();

        let build_list_start = Instant::now();
        self.renderer.borrow_mut().clear();
        let scale_factor = self.effective_scale_factor();

        self.draw_transformed(
            &mut *renderer_clone.borrow_mut(),
            resource_manager.clone(),
            scale_factor,
            text_context,
        );
        let build_list = build_list_start.elapsed();

        let debug_overlay_start = Instant::now();
        self.perf_stats
            .draw(&mut *renderer_clone.borrow_mut(), text_context, scale_factor);
        let debug_overlay = debug_overlay_start.elapsed();

        self.winit_window.clone().unwrap().pre_present_notify();

        let (sort, prepare, submit) = {
            let renderer = renderer_clone.clone();
            let sort_start = Instant::now();
            renderer.borrow_mut().sort_render_list();
            let sort = sort_start.elapsed();

            let window = Rectangle::new(
                0.0,
                0.0,
                renderer.borrow().surface_width(),
                renderer.borrow().surface_height(),
            );
            let prepare_start = Instant::now();
            renderer.borrow_mut().prepare(resource_manager.clone(), window);
            let prepare = prepare_start.elapsed();
            let submit_start = Instant::now();
            renderer.borrow_mut().submit(resource_manager.clone());
            let submit = submit_start.elapsed();

            (sort, prepare, submit)
        };

        if self.perf_stats.is_enabled() {
            self.request_redraw();
        }

        RenderStats::new(total_start.elapsed(), build_list, debug_overlay, sort, prepare, submit)
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
