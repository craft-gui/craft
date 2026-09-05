//! Stores one or more elements.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use gummy::AvailableSpace;

use retgui_logging::info;

use retgui_primitives::geometry::{Point, Rectangle, Size};

use retgui_renderer::RendererType;
use retgui_renderer::blank_renderer::BlankRenderer;
use retgui_renderer::renderer::{Renderer, Screenshot};

use retgui_resource_manager::ResourceManager;

#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, winit::platform::web::WindowAttributesWeb};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use winit::event::MouseScrollDelta;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window as WinitWindow, WindowAttributes};

use crate::Color;
use crate::accessibility::RetGuiAccessTree;
use crate::app::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::{DynElement, Element, ElementNode, Elements, scrollable};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{EventKind, KeyboardEvent, PointerScrollEvent, PointerType};
use crate::layout::GummyTree;
use crate::perf_stats::{LayoutStats, PerfStats, RenderStats};
use crate::text::text_context::TextContext;

pub type WindowConstructor = Box<dyn FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>;

#[derive(Clone, Copy)]
pub struct Window {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub(crate) struct WindowNode {
    /// The physical window size from winit.
    pub(crate) window_size: Size<f32>,
    pub(crate) renderer: Box<dyn Renderer>,

    // Will be empty when paused.
    pub(crate) winit_window: Option<Arc<dyn WinitWindow>>,
    pub(crate) headless: bool,
    redraw_requested: Arc<AtomicBool>,

    pub(crate) access_tree: RetGuiAccessTree,
    pub(crate) pointer_capture: PointerCapture,

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
    pub(crate) modifiers: ModifiersState,
    pub(crate) ime_composing: bool,
}

impl Clone for WindowNode {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Element for Window {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for WindowNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for WindowNode {
    fn window_pointer_capture(&mut self) -> Option<&mut PointerCapture> {
        Some(&mut self.pointer_capture)
    }

    fn request_window_redraw(&self) {
        self.request_redraw();
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
        &self,
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(elements, self, event);
    }

    fn deep_clone(&self, _elements: &mut Elements) -> DynElement {
        todo!()
    }
}

impl Window {
    pub fn new_advanced<F>(elements: &mut Elements, window_fn: F, renderer_type: RendererType) -> Self
    where
        F: FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow> + 'static,
    {
        let inner = WindowNode::insert(elements, Some(window_fn), None, renderer_type);

        Window { inner }
    }

    pub fn new(elements: &mut Elements, title: &str) -> Self {
        let inner = WindowNode::insert(
            elements,
            None::<fn(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>,
            Some(title),
            RendererType::default(),
        );

        Window { inner }
    }

    pub fn new_with_renderer(elements: &mut Elements, title: &str, renderer_type: RendererType) -> Self {
        let inner = WindowNode::insert(
            elements,
            None::<fn(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>,
            Some(title),
            renderer_type,
        );

        Window { inner }
    }

    pub fn screenshot(&self, elements: &mut Elements) -> Screenshot {
        elements.get_as_mut::<WindowNode>(self.inner).screenshot()
    }

    pub fn close(&self, elements: &Elements) {
        elements.get_as::<WindowNode>(self.inner).close();
    }

    pub fn winit_window(&self, elements: &Elements) -> Option<Arc<dyn winit::window::Window>> {
        elements.get_as::<WindowNode>(self.inner).winit_window()
    }

    pub fn set_winit_window(&self, elements: &mut Elements, window: Option<Arc<dyn WinitWindow>>) {
        elements.get_as_mut::<WindowNode>(self.inner).set_winit_window(window)
    }

    pub fn set_scale_factor(&self, elements: &mut Elements, scale_factor: f64) {
        elements.dispatch_mut(self.inner, |window, elements| {
            window.set_scale_factor(elements, scale_factor)
        });
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self, elements: &Elements) -> f64 {
        elements.get_as::<WindowNode>(self.inner).effective_scale_factor()
    }

    /// Get the logical size of the window.
    pub fn window_size(&self, elements: &Elements) -> Size<f32> {
        elements.get_as::<WindowNode>(self.inner).window_size()
    }

    pub fn zoom_scale_factor(&self, elements: &Elements) -> f64 {
        elements.get_as::<WindowNode>(self.inner).zoom_scale_factor()
    }

    pub fn on_request_redraw(&self, retgui_app: &mut App) {
        let mut elements = std::mem::take(&mut retgui_app.elements);
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .on_request_redraw(retgui_app, elements)
        });
        retgui_app.elements = elements;
    }

    pub(crate) fn request_redraw(&self, elements: &Elements) {
        elements.get_as::<WindowNode>(self.inner).request_redraw();
    }

    pub(crate) fn redraw_requested(&self, elements: &Elements) -> bool {
        elements.get_as::<WindowNode>(self.inner).redraw_requested()
    }

    pub fn zoom_in(&self, elements: &mut Elements) {
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .zoom_in(elements)
        })
    }

    pub fn zoom_out(&self, elements: &mut Elements) {
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .zoom_out(elements)
        })
    }

    pub(crate) fn mouse_position(&self, elements: &Elements) -> Option<Point> {
        elements.get_as::<WindowNode>(self.inner).mouse_position()
    }

    pub(crate) fn on_resize(&self, elements: &mut Elements, new_size: Size<f32>) {
        let (gummy_tree, nodes) = elements.disjoint_borrow_layout_and_elements();
        nodes
            .get_as_mut::<WindowNode>(self.inner)
            .on_resize(gummy_tree, new_size)
    }

    pub(crate) fn set_mouse_position(&self, elements: &mut Elements, point: Option<Point>) {
        elements.get_as_mut::<WindowNode>(self.inner).set_mouse_position(point)
    }

    pub(crate) fn on_redraw(
        &self,
        elements: &mut Elements,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) {
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .on_redraw(elements, text_context, resource_manager)
        })
    }

    pub(crate) fn create(
        &self,
        retgui_app: &mut App,
        elements: &mut Elements,
        event_loop: Option<&dyn ActiveEventLoop>,
    ) {
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .create(retgui_app, elements, event_loop, self.inner)
        });
    }

    pub(crate) fn on_scale_factor_changed(&self, elements: &mut Elements, scale_factor: f64) {
        elements.dispatch_mut(self.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowNode>()
                .unwrap()
                .on_scale_factor_changed(elements, scale_factor)
        });
    }

    pub(crate) fn on_focused(&self, elements: &Elements, focused: bool) {
        elements.get_as::<WindowNode>(self.inner).on_focused(elements, focused);
    }
}

impl WindowNode {
    pub fn insert<F>(
        elements: &mut Elements,
        f: Option<F>,
        title: Option<&str>,
        renderer_type: RendererType,
    ) -> DynElement
    where
        F: FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow> + 'static,
    {
        let perf_stats = PerfStats::new(elements);
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(Self {
                element_data: ElementData::new(me, true, access_tree.clone()),
                window_size: Default::default(),
                scale_factor: 1.0,
                zoom_scale_factor: 1.0,
                perf_stats,
                mouse_positon: None,
                renderer: Box::new(BlankRenderer::default()),
                winit_window: None,
                headless: false,
                redraw_requested: Arc::new(AtomicBool::new(false)),
                access_tree,
                advanced_window_fn: f.map(|f| Box::new(f) as WindowConstructor),
                title: title.map(|title| title.to_string()),
                renderer_type,
                pointer_capture: Default::default(),
                modifiers: Default::default(),
                ime_composing: false,
            })
        });

        {
            let inner_mut = elements.get_as_mut::<Self>(inner);
            inner_mut.element_data.window = Some(inner);
            inner_mut.element_data.redraw_signal = Some(inner_mut.redraw_requested.clone());
            inner_mut.element_data.set_accessibility_role(issho::Role::Window);
            if let Some(title) = inner_mut.title.clone() {
                inner_mut.element_data.set_accessibility_name(title);
            }
        }
        elements.create_layout_node(inner, None);

        elements.with_window_manager(|window_manager, _| {
            window_manager.add_window(Window { inner });
        });

        inner
    }

    pub fn request_redraw(&self) {
        self.redraw_requested.store(true, Ordering::Relaxed);
        if let Some(winit_window) = &self.winit_window {
            winit_window.request_redraw();
        }
    }

    pub(crate) fn redraw_requested(&self) -> bool {
        self.redraw_requested.load(Ordering::Relaxed)
    }

    pub(crate) fn on_focused(&self, elements: &Elements, focused: bool) {
        {
            let root = self
                .element_data
                .access_root
                .expect("window accessibility root is not attached");
            if focused {
                let focus = elements.focus.and_then(|focus| {
                    elements
                        .contains(focus)
                        .then(|| {
                            let element = elements.get(focus);
                            let data = element.element_data();
                            let belongs_to_window =
                                data.access_tree.ptr_eq(&self.access_tree) && data.access_root == Some(root);
                            belongs_to_window.then_some(data.access_key).flatten()
                        })
                        .flatten()
                });
                self.access_tree.set_focus(root, Some(focus.unwrap_or(root)));
            } else {
                self.access_tree.set_focus(root, None);
            }
        }
    }

    pub fn winit_window(&self) -> Option<Arc<dyn winit::window::Window>> {
        self.winit_window.clone()
    }

    pub fn set_winit_window(&mut self, window: Option<Arc<dyn WinitWindow>>) {
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

    pub fn update_zoom(&mut self, elements: &mut Elements) {
        let scale_factor = self.effective_scale_factor();
        self.set_scale_factor(elements, scale_factor);
    }

    pub fn on_request_redraw(&mut self, retgui_app: &mut App, elements: &mut Elements) {
        self.on_redraw(
            elements,
            &mut retgui_app.text_context,
            retgui_app.resource_manager.clone(),
        );
    }

    pub(crate) fn zoom_in(&mut self, elements: &mut Elements) {
        self.zoom_scale_factor += 0.01;
        self.update_zoom(elements);
    }

    pub(crate) fn zoom_out(&mut self, elements: &mut Elements) {
        let zoom_scale_factor = (self.zoom_scale_factor - 0.01).max(1.0);
        if self.zoom_scale_factor == zoom_scale_factor {
            return;
        }
        self.zoom_scale_factor = zoom_scale_factor;
        self.update_zoom(elements);
    }

    pub(crate) fn maybe_zoom(&mut self, elements: &mut Elements, pointer_scroll_update: &PointerScrollEvent) -> bool {
        if self.modifiers.control_key() && pointer_scroll_update.pointer.pointer_type == PointerType::Mouse {
            let y: f32 = match pointer_scroll_update.delta {
                MouseScrollDelta::LineDelta(_, y) => y,
                MouseScrollDelta::PixelDelta(physical) => physical.y as f32,
                _ => 0.0,
            };
            if y < 0.0 {
                self.zoom_out(elements);
            } else {
                self.zoom_in(elements);
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn maybe_zoom_keyboard(&mut self, elements: &mut Elements, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.modifiers.control_key() {
            if keyboard_input.key == "=" {
                self.zoom_in(elements);
                return true;
            } else if keyboard_input.key == "-" {
                self.zoom_out(elements);
                return true;
            }
        }
        false
    }

    pub(crate) fn tab_navigation_target(
        &self,
        elements: &Elements,
        keyboard_input: &KeyboardEvent,
    ) -> Option<DynElement> {
        if !keyboard_input.state.is_pressed()
            || keyboard_input.key != Key::Named(NamedKey::Tab)
            || keyboard_input.modifiers.control_key()
            || keyboard_input.modifiers.alt_key()
            || keyboard_input.modifiers.meta_key()
        {
            return None;
        }

        let mut navigation_elements = Vec::new();
        collect_tab_navigation_elements(elements, &self.element_data.children, &mut navigation_elements);
        if navigation_elements.is_empty() {
            return None;
        }

        let current_focus = elements.focus;
        let current_index = current_focus
            .as_ref()
            .and_then(|current| navigation_elements.iter().position(|(element, _)| element == current));
        let len = navigation_elements.len();
        let next_index = if let Some(current_index) = current_index {
            (1..=len)
                .map(|offset| {
                    if keyboard_input.modifiers.shift_key() {
                        (current_index + len - offset) % len
                    } else {
                        (current_index + offset) % len
                    }
                })
                .find(|index| navigation_elements[*index].1)
        } else if keyboard_input.modifiers.shift_key() {
            navigation_elements.iter().rposition(|(_, focusable)| *focusable)
        } else {
            navigation_elements.iter().position(|(_, focusable)| *focusable)
        };
        next_index.map(|index| navigation_elements[index].0)
    }

    pub(crate) fn maybe_toggle_perf_stats(&mut self, elements: &mut Elements, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.repeat || !keyboard_input.state.is_pressed() {
            return false;
        }

        if keyboard_input.key != Key::Named(NamedKey::F3) {
            return false;
        }

        self.perf_stats.toggle(elements, &mut *self.renderer);
        self.request_redraw();
        true
    }

    pub(crate) fn perf_stats_enabled(&self) -> bool {
        self.perf_stats.is_enabled()
    }

    pub(crate) fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub(crate) fn zoom_scale_factor(&self) -> f64 {
        self.zoom_scale_factor
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.mouse_positon
    }

    pub(crate) fn on_resize(&mut self, gummy_tree: &mut GummyTree, new_size: Size<f32>) {
        if self.window_size.width == new_size.width && self.window_size.height == new_size.height {
            return;
        }
        self.mark_dirty(gummy_tree);

        self.window_size = new_size;
        self.resize_renderer_surface();
        self.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn on_renderer_created(
        &mut self,
        gummy_tree: &mut GummyTree,
        renderer: Box<dyn Renderer>,
        new_size: Size<f32>,
    ) {
        self.renderer = renderer;
        if self.window_size.width != new_size.width || self.window_size.height != new_size.height {
            self.mark_dirty(gummy_tree);
            self.window_size = new_size;
        }
        self.resize_renderer_surface();
        self.request_redraw();
    }

    fn resize_renderer_surface(&mut self) {
        let size = self.window_size;
        self.renderer.resize_surface(size.width.max(1.0), size.height.max(1.0));
        self.renderer
            .set_cull(Some(Rectangle::new(0.0, 0.0, size.width, size.height)));
    }

    pub(crate) fn set_mouse_position(&mut self, point: Option<Point>) {
        self.mouse_positon = point;
    }

    pub(crate) fn on_redraw(
        &mut self,
        elements: &mut Elements,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) {
        self.redraw_requested.store(false, Ordering::Relaxed);

        let frame_start = Instant::now();
        self.renderer.surface_set_clear_color(Color::WHITE);

        let layout_stats = self.layout_window(elements, text_context, resource_manager.clone());

        let render_stats = self.draw_window(elements, text_context, resource_manager);
        self.perf_stats
            .update_stats(frame_start.elapsed(), layout_stats, render_stats);
    }

    pub(crate) fn on_scale_factor_changed(&mut self, elements: &mut Elements, scale_factor: f64) {
        if self.scale_factor == scale_factor {
            return;
        }
        self.scale_factor = scale_factor;
        self.set_scale_factor(elements, self.effective_scale_factor());
    }

    pub(crate) fn create(
        &mut self,
        retgui_app: &mut App,
        elements: &mut Elements,
        event_loop: Option<&dyn ActiveEventLoop>,
        window: DynElement,
    ) {
        let Some(event_loop) = event_loop else {
            if self.headless {
                return;
            }

            self.headless = true;
            self.renderer = self
                .renderer_type
                .create_headless(self.window_size.width, self.window_size.height);
            self.resize_renderer_surface();
            self.on_request_redraw(retgui_app, elements);
            return;
        };

        self.headless = false;

        let winit_window: Arc<dyn WinitWindow> = Arc::from(if let Some(window_fn) = &mut self.advanced_window_fn {
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

                window_attributes
                    .with_platform_attributes(Box::new(WindowAttributesWeb::default().with_canvas(Some(canvas))))
            };

            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window")
        });
        self.set_winit_window(Some(winit_window.clone()));
        self.on_scale_factor_changed(elements, winit_window.scale_factor());

        let renderer_type = self.renderer_type;
        cfg_select! {
            not(target_arch = "wasm32") => {
                let _ = window;
                let renderer = retgui_app.runtime.tokio_runtime_mut().block_on(async {
                    let renderer: Box<dyn Renderer> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                self.renderer = renderer;
                info!("Created renderer")
            }
            _ => {
                let window_copy_2 = winit_window.clone();
                let created_renderer_sender = retgui_app.created_renderer_sender.clone();
                retgui_app.runtime.runtime_spawn(async move {
                    let renderer: Box<dyn Renderer> = renderer_type.create(window_copy_2.clone()).await;
                    let size = Size::new(
                        window_copy_2.surface_size().width as f32,
                        window_copy_2.surface_size().height as f32,
                    );
                    let _ = created_renderer_sender.send(crate::app::CreatedRenderer {
                        window,
                        renderer,
                        size,
                    });
                    info!("Created renderer")
                });
            }
        }

        {
            self.on_request_redraw(retgui_app, elements);
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

    fn layout_window(
        &mut self,
        elements: &mut Elements,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) -> LayoutStats {
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

        let (gummy_tree, nodes) = elements.disjoint_borrow_layout_and_elements();
        let root_dirty = gummy_tree.is_layout_dirty(root_node);

        if root_dirty {
            let compute_start = Instant::now();
            gummy_tree.compute_layout_with_nodes(
                root_node,
                available_space,
                nodes,
                text_context,
                resource_manager.clone(),
            );
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
                    nodes
                        .get_mut(owner)
                        .apply_layout(gummy_tree, &mut layout_order, text_context, sf);
                }
            }
            gummy_tree.apply_layout(root_node);
            apply = apply_start.elapsed();
        }

        LayoutStats::new(total_start.elapsed(), compute, apply)
    }

    fn draw_window(
        &mut self,
        elements: &mut Elements,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) -> RenderStats {
        let total_start = Instant::now();
        let mut renderer = std::mem::replace(&mut self.renderer, Box::new(BlankRenderer::default()));

        let build_list_start = Instant::now();
        renderer.clear();
        let scale_factor = self.effective_scale_factor();

        self.draw_transformed(
            elements,
            &mut *renderer,
            resource_manager.clone(),
            scale_factor,
            text_context,
        );
        let build_list = build_list_start.elapsed();

        let debug_overlay_start = Instant::now();
        self.perf_stats
            .draw(elements, &mut *renderer, text_context, scale_factor);
        let debug_overlay = debug_overlay_start.elapsed();

        if let Some(winit_window) = &self.winit_window {
            winit_window.pre_present_notify();
        }

        let (sort, prepare, submit) = {
            let sort_start = Instant::now();
            renderer.sort_render_list();
            let sort = sort_start.elapsed();

            let window = Rectangle::new(0.0, 0.0, renderer.surface_width(), renderer.surface_height());
            let prepare_start = Instant::now();
            renderer.prepare(resource_manager.clone(), window);
            let prepare = prepare_start.elapsed();
            let submit_start = Instant::now();
            renderer.submit(resource_manager.clone());
            let submit = submit_start.elapsed();

            (sort, prepare, submit)
        };

        if self.perf_stats.is_enabled() {
            self.request_redraw();
        }
        self.renderer = renderer;

        RenderStats::new(total_start.elapsed(), build_list, debug_overlay, sort, prepare, submit)
    }

    fn screenshot(&mut self) -> Screenshot {
        self.renderer.screenshot()
    }

    fn close(&self) {
        todo!("programmatic window closing is not implemented")
    }
}

fn collect_tab_navigation_elements(
    elements: &Elements,
    children: &[DynElement],
    navigation_elements: &mut Vec<(DynElement, bool)>,
) {
    for child in children {
        let (is_visible, is_keyboard_focusable, grandchildren) = {
            let element = elements.get(*child);
            (
                element.is_visible(),
                element.is_keyboard_focusable(),
                element.element_data().children.clone(),
            )
        };

        if !is_visible {
            continue;
        }
        navigation_elements.push((*child, is_keyboard_focusable));
        collect_tab_navigation_elements(elements, &grandchildren, navigation_elements);
    }
}
