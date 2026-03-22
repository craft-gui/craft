//! Stores one or more elements.

use std::any::Any;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use {accesskit::{Action, Role, TreeUpdate}, accesskit_winit::Adapter};

use craft_logging::info;

use craft_primitives::geometry::{Affine, Point, Rectangle, Size};

use craft_renderer::renderer::{Renderer, Screenshot};
use craft_renderer::{RenderList, RendererType};

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

#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, winit::platform::web::WindowAttributesExtWebSys};

#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::accessibility::{access_handler::CraftAccessHandler, activation_handler::CraftActivationHandler, deactivation_handler::CraftDeactivationHandler};
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::app::FOCUS;
use crate::app::{App, TAFFY_TREE, WINDOW_MANAGER, queue_window_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
#[cfg(target_arch = "wasm32")]
use crate::events::internal::InternalMessage;
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;

pub type WindowConstructor = Box<dyn FnMut(&ActiveEventLoop) -> WinitWindow>;

#[cfg(not(target_arch = "wasm32"))]
type RendererBox = Box<dyn Renderer>;
#[cfg(target_arch = "wasm32")]
type RendererBox = Box<dyn Renderer>;

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
    pub(crate) renderer: Option<RendererBox>,
    pub(crate) render_list: Rc<RefCell<RenderList>>,

    // Will be empty when paused.
    pub(crate) winit_window: Option<Arc<WinitWindow>>,

    // Will be empty when paused.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) accesskit_adapter: Option<Adapter>,
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

impl AsElement for Window {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
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
    fn pointer_capture(&self) -> Rc<RefCell<PointerCapture>> {
        self.pointer_capture.clone()
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

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        draw_generic_container(self, renderer, text_context, scale_factor);
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(&mut self, tree: &mut TreeUpdate, parent_index: Option<usize>, scale_factor: f64) {
        let current_node_id = accesskit::NodeId(self.element_data.internal_id);

        let mut current_node = accesskit::Node::new(Role::Window);
        if !self.element_data.on_pointer_button_up.is_empty() {
            current_node.set_role(Role::Button);
            current_node.add_action(Action::Click);
        }

        let padding_box = self
            .element_data
            .layout
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });

        let current_index = tree.nodes.len(); // The current node is the last one added.

        if let Some(parent_index) = parent_index {
            let parent_node = tree.nodes.get_mut(parent_index).unwrap();
            parent_node.1.push_child(current_node_id);
        }

        tree.nodes.push((current_node_id, current_node));

        for child in self.element_data.children.iter_mut() {
            child
                .borrow_mut()
                .compute_accessibility_tree(tree, Some(current_index), scale_factor);
        }
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        scrollable::handle_scroll_logic(self, message, event);
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
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

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub fn on_request_redraw(&self, craft_app: &mut App) -> Option<TreeUpdate> {
        self.inner.borrow_mut().on_request_redraw(craft_app)
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(any(not(feature = "accesskit"), target_arch = "wasm32"))]
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

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) fn compute_accessibility_tree_window(&self) -> TreeUpdate {
        self.inner.borrow_mut().compute_accessibility_tree_window()
    }

    pub(crate) fn create(&self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        self.inner.borrow_mut().create(craft_app, event_loop)
    }

    pub(crate) fn on_scale_factor_changed(&self, scale_factor: f64) {
        self.inner.borrow_mut().on_scale_factor_changed(scale_factor);
    }
}

impl WindowInternal {
    pub fn new<F>(f: Option<F>, title: Option<&str>, renderer_type: RendererType) -> Rc<RefCell<Self>>
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), true),
                window_size: Default::default(),
                scale_factor: 1.0,
                zoom_scale_factor: 1.0,
                mouse_positon: None,
                renderer: None,
                render_list: Rc::new(RefCell::new(RenderList::new())),
                winit_window: None,
                #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
                accesskit_adapter: None,
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

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub fn on_request_redraw(&mut self, craft_app: &mut App) -> Option<TreeUpdate> {
        self.on_redraw(
            craft_app.text_context.as_mut().unwrap(),
            craft_app.resource_manager.clone(),
        );

        let tree_update = self.compute_accessibility_tree_window();
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            accesskit_adapter.update_if_active(|| tree_update);
            None
        } else {
            Some(tree_update)
        }
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(any(not(feature = "accesskit"), target_arch = "wasm32"))]
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
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize_surface(new_size.width.max(1.0), new_size.height.max(1.0));
        }
        self.window_size = new_size;
        let size = self.window_size;
        self.render_list
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
        if self.renderer.is_none() {
            return;
        }

        self.renderer.as_mut().unwrap().surface_set_clear_color(Color::WHITE);

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

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    let renderer = craft_app.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Box<dyn Renderer> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                self.renderer = Some(renderer);
                info!("Created renderer")
            } else {
                let window_copy_2 = winit_window.clone();
                craft_app.runtime.spawn(async move {
                    let renderer: Box<dyn Renderer> = renderer_type.create(window_copy_2.clone()).await;
                    WASM_QUEUE.with_borrow_mut(|wasm_queue| {
                        wasm_queue.push(InternalMessage::RendererCreated(window_copy_2.clone(), renderer));
                    });
                    info!("Created renderer")
                });
            }
        }

        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        {
            let action_handler = CraftAccessHandler {};
            let deactivation_handler = CraftDeactivationHandler::new();

            let tree_update = self.on_request_redraw(craft_app);

            let craft_activation_handler = CraftActivationHandler::new(tree_update);
            self.accesskit_adapter = Some(Adapter::with_direct_handlers(
                event_loop,
                &winit_window,
                craft_activation_handler,
                action_handler,
                deactivation_handler,
            ));
        }

        winit_window.set_visible(true);
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) fn compute_accessibility_tree_window(&mut self) -> TreeUpdate {
        let window_accesskit_id = self.element_data.internal_id;
        let tree = accesskit::Tree {
            root: accesskit::NodeId(window_accesskit_id),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let focus_id = FOCUS.with_borrow_mut(|focus| {
            if let Some(focus) = focus
                && let Some(focus) = focus.upgrade()
            {
                return focus.borrow().element_data().internal_id;
            }
            window_accesskit_id
        });

        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(focus_id),
        };

        let scale_factor = self.winit_window.as_mut().unwrap().scale_factor();
        self.compute_accessibility_tree(&mut tree_update, None, scale_factor);

        tree_update
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
                    None,
                    sf,
                );
                taffy_tree.apply_layout(root_node);
            }
            //}
        });

        root_node
    }

    fn draw_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) {
        let render_list = self.render_list.clone();
        render_list.borrow_mut().deref_mut().clear();
        self.draw(
            render_list.borrow_mut().deref_mut(),
            text_context,
            self.effective_scale_factor(),
        );

        self.winit_window.clone().unwrap().pre_present_notify();
        self.renderer
            .as_mut()
            .unwrap()
            .sort_and_cull_render_list(render_list.borrow_mut().deref_mut());

        let window = Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.renderer.as_mut().unwrap().surface_width(),
            height: self.renderer.as_mut().unwrap().surface_height(),
        };
        self.renderer.as_mut().unwrap().prepare_render_list(
            self.render_list.borrow_mut().deref_mut(),
            resource_manager.clone(),
            window,
        );

        self.renderer.as_mut().unwrap().submit(resource_manager.clone());
    }

    fn screenshot(&self) -> Screenshot {
        self.renderer.as_ref().unwrap().screenshot()
    }

    fn close(&self) {
        if let Some(winit_window) = &self.winit_window {
            queue_window_event(winit_window.id(), WindowEvent::CloseRequested);
        }
    }
}
