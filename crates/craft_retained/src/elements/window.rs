//! Stores one or more elements.

use std::any::Any;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use accesskit::TreeUpdate;
use craft_primitives::geometry::{Rectangle, Size};
use craft_renderer::RenderList;
use craft_renderer::renderer::Renderer;
use craft_resource_manager::ResourceManager;
use kurbo::{Affine, Point};
use peniko::Color;
use taffy::{AvailableSpace, NodeId};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowAttributes};
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use {accesskit::{Action, Role}, accesskit_winit::Adapter};

use crate::RendererBox;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::accessibility::{access_handler::CraftAccessHandler, activation_handler::CraftActivationHandler, deactivation_handler::CraftDeactivationHandler};
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::app::FOCUS;
use crate::app::{App, TAFFY_TREE, WINDOW_MANAGER};
use crate::elements::core::{ElementInternals, resolve_clip_for_scrollable};
use crate::elements::element::AsElement;
use crate::elements::element_data::ElementData;
use crate::elements::{Element, ElementImpl, scrollable};
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct Window {
    pub inner: Rc<RefCell<WindowInternal>>,
}

pub type WindowConstructor = Box<dyn FnMut(&ActiveEventLoop) -> WinitWindow>;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct WindowInternal {
    element_data: ElementData,
    /// The physical window size from winit.
    pub(crate) window_size: Size<f32>,
    /// The window's scale factor from winit.
    scale_factor: f64,
    /// Zoom scale factor.
    zoom_scale_factor: f64,
    mouse_positon: Option<Point>,
    pub(crate) renderer: Option<RendererBox>,
    pub(crate) render_list: Rc<RefCell<RenderList>>,

    // Will be empty when paused.
    pub(crate) winit_window: Option<Arc<WinitWindow>>,

    // Will be empty when paused.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) accesskit_adapter: Option<Adapter>,

    advanced_window_fn: Option<WindowConstructor>,
}

impl Window {
    pub fn new_advanced<F>(f: F) -> Self
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        let inner = WindowInternal::new(Some(f));

        inner.borrow_mut().element_data.create_layout_node(None);

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.add_window(Window {
                inner: inner.clone(),
            });
        });

        Window { inner }
    }

    pub fn new() -> Self {
        let inner = WindowInternal::new(None::<fn(&ActiveEventLoop) -> WinitWindow>);

        inner.borrow_mut().element_data.create_layout_node(None);

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.add_window(Window {
                inner: inner.clone(),
            });
        });

        Window { inner }
    }
}

impl Window {
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

    pub(crate) fn zoom_in(&self) {
        self.inner.borrow_mut().zoom_in()
    }

    pub(crate) fn zoom_out(&self) {
        self.inner.borrow_mut().zoom_out()
    }

    pub fn zoom_scale_factor(&self) -> f64 {
        self.inner.borrow().zoom_scale_factor()
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

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub fn on_request_redraw(&self, craft_app: &mut App) -> Option<TreeUpdate> {
        self.inner.borrow_mut().on_request_redraw(craft_app)
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) fn compute_accessibility_tree_window(&self) -> TreeUpdate {
        self.inner.borrow_mut().compute_accessibility_tree_window()
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(any(not(feature = "accesskit"), target_arch = "wasm32"))]
    pub fn on_request_redraw(&self, craft_app: &mut App) {
        self.inner.borrow_mut().on_request_redraw(craft_app)
    }

    pub(crate) fn create(&self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        self.inner.borrow_mut().create(craft_app, event_loop)
    }

    pub(crate) fn on_scale_factor_changed(&self, scale_factor: f64) {
        self.inner.borrow_mut().on_scale_factor_changed(scale_factor);
    }
}

impl WindowInternal {
    pub fn new<F>(f: Option<F>) -> Rc<RefCell<Self>>
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
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
            })
        })
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

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_scale_factor += 0.01;
    }

    pub(crate) fn zoom_out(&mut self) {
        self.zoom_scale_factor = (self.zoom_scale_factor - 0.01).max(1.0);
    }

    pub(crate) fn zoom_scale_factor(&self) -> f64 {
        self.zoom_scale_factor
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.mouse_positon
    }

    pub(crate) fn on_resize(&mut self, new_size: Size<f32>) {
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            taffy_tree.mark_dirty(self.element_data.layout_item.taffy_node_id.unwrap());
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
            self.window.as_ref().unwrap().request_redraw();
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

    fn layout_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) -> NodeId {
        let root_node = self
            .element_data
            .layout_item
            .taffy_node_id
            .expect("A root element must have a layout node.");

        let window_size = self.window_size();
        let available_space: taffy::Size<AvailableSpace> = taffy::Size {
            width: AvailableSpace::Definite(window_size.width as f32),
            height: AvailableSpace::Definite(window_size.height as f32),
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
                    None,
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
            None,
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

    pub(crate) fn create(&mut self, craft_app: &mut App, event_loop: &ActiveEventLoop) {
        let winit_window: Arc<WinitWindow> = Arc::new(if let Some(window_fn) = &mut self.advanced_window_fn {
            (*window_fn)(event_loop)
        } else {
            event_loop
                .create_window(WindowAttributes::default().with_visible(false))
                .expect("Failed to create window")
        });
        self.set_winit_window(Some(winit_window.clone()));
        self.on_scale_factor_changed(winit_window.scale_factor());

        let renderer_type = craft_app.craft_options.renderer;

        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                    let renderer = craft_app.runtime.borrow_tokio_runtime().block_on(async {
                        let renderer: Box<dyn Renderer> = renderer_type.create(winit_window.clone()).await;
                    renderer
                });
                self.renderer = Some(renderer);
            } else {
                let app_sender = craft_state.app_sender.clone();
                let window_copy_2 = window_copy.clone();
                craft_state.runtime.spawn(async move {
                    let renderer: Box<dyn Renderer> = renderer_type.create(window_copy).await;
                    app_sender
                        .send(InternalMessage::RendererCreated(window_copy_2, renderer))
                        .await
                        .expect("Failed to send RendererCreated message");
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

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    pub(crate) fn compute_accessibility_tree_window(&mut self) -> TreeUpdate {
        let window_accesskit_id = self.element_data.internal_id;
        let tree = accesskit::Tree {
            root: accesskit::NodeId(window_accesskit_id),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let focus_id = FOCUS.with_borrow_mut(|focus| {
            if let Some(focus) = focus {
                if let Some(focus) = focus.upgrade() {
                    return focus.borrow().element_data().internal_id;
                }
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

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(any(not(feature = "accesskit"), target_arch = "wasm32"))]
    pub fn on_request_redraw(&mut self, craft_app: &mut App) {
        self.on_redraw(
            craft_app.text_context.as_mut().unwrap(),
            craft_app.resource_manager.clone(),
        );
    }
}

impl Element for Window {}

impl AsElement for Window {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementImpl>> {
        self.inner.clone()
    }
}

impl crate::elements::core::ElementData for WindowInternal {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementImpl for WindowInternal {
    fn push(&mut self, child: Rc<RefCell<dyn ElementImpl>>) {
        let me: Weak<RefCell<dyn ElementImpl>> = self.element_data.me.clone();
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(child.clone());

        // Add the children's taffy node.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data.layout_item.taffy_node_id.unwrap();
            let child_id = child.borrow().element_data().layout_item.taffy_node_id;
            if let Some(child_id) = child_id {
                taffy_tree.add_child(parent_id, child_id);
            }
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for WindowInternal {
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout_item.taffy_node_id.unwrap();
        let layout = taffy_tree.layout(node);
        let has_new_layout = taffy_tree.get_has_new_layout(node);

        let dirty = has_new_layout
            || transform != self.element_data.layout_item.get_transform()
            || position != self.element_data.layout_item.position;
        self.element_data.layout_item.has_new_layout = has_new_layout;

        if dirty {
            self.resolve_box(position, transform, layout, z_index);
            self.apply_borders(scale_factor);
            // For scroll changes from taffy;
            self.element_data.apply_scroll(layout);
            self.apply_clip(clip_bounds);
            self.element_data.scroll_state.as_mut().unwrap().mark_old();
        }

        // For manual scroll updates.
        if !dirty
            && self
                .element_data
                .scroll_state
                .map(|scroll_state| scroll_state.is_new())
                .unwrap_or_default()
        {
            self.element_data.apply_scroll(layout);
            self.element_data.scroll_state.as_mut().unwrap().mark_old();
        }

        if has_new_layout {
            taffy_tree.mark_seen(node);
        }

        let scroll_y = self.element_data.scroll().map_or(0.0, |s| s.scroll_y() as f64);
        let child_transform = Affine::translate((0.0, -scroll_y));

        self.apply_layout_children(
            taffy_tree,
            z_index,
            transform * child_transform,
            pointer,
            text_context,
            scale_factor,
            false,
        )
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        scale_factor: f64,
    ) {
        if !self.is_visible() {
            return;
        }
        self.add_hit_testable(renderer, true, scale_factor);

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);

        /*if self.element_data.layout_item.has_new_layout {
            renderer.draw_rect_outline(self.element_data.layout_item.computed_box_transformed.padding_rectangle(), rgba(255, 0, 0, 100), 5.0);
        }*/

        self.maybe_start_layer(renderer, scale_factor);
        self.draw_children(renderer, text_context, pointer, scale_factor);
        self.maybe_end_layer(renderer);

        self.draw_scrollbar(renderer, scale_factor);
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let current_node_id = accesskit::NodeId(self.element_data.internal_id);

        let mut current_node = accesskit::Node::new(Role::Window);
        if !self.element_data.on_pointer_button_up.is_empty() {
            current_node.set_role(Role::Button);
            current_node.add_action(Action::Click);
        }

        let padding_box = self
            .element_data
            .layout_item
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
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        scrollable::on_scroll_events(self, message, event);
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }
}
