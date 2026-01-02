//! Stores one or more elements.

use crate::app::{TAFFY_TREE, WINDOW_MANAGER};
use crate::elements::core::{resolve_clip_for_scrollable, ElementInternals};
use crate::elements::element_data::ElementData;
use crate::elements::{scrollable, Element};
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::{Rectangle, Size};
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use std::any::Any;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use peniko::Color;
use taffy::{AvailableSpace, NodeId};
use winit::window::Window as WinitWindow;
use crate::RendererBox;
use craft_resource_manager::ResourceManager;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct Window {
    element_data: ElementData,
    pub(crate) winit_window: Option<Arc<WinitWindow>>,
    /// The physical window size from winit.
    pub(crate) window_size: Size<f32>,
    /// The window's scale factor from winit.
    scale_factor: f64,
    /// Zoom scale factor.
    zoom_scale_factor: f64,
    mouse_positon: Option<Point>,
    pub(crate) renderer: Option<RendererBox>,
    pub(crate) render_list: Rc<RefCell<RenderList>>,
}

impl Window {
    pub fn new() -> Rc<RefCell<Self>> {
        let me = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), true),
                winit_window: None,
                window_size: Default::default(),
                scale_factor: 1.0,
                zoom_scale_factor: 1.0,
                mouse_positon: None,
                renderer: None,
                render_list: Rc::new(RefCell::new(RenderList::new())),
            })
        });

        me.borrow_mut().element_data.create_layout_node(None);

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.add_window(me.clone());
        });

        me
    }

    pub fn winit_window(&self) -> Option<Arc<winit::window::Window>> {
        self.winit_window.clone()
    }

    pub fn set_winit_window(&mut self, window: Arc<WinitWindow>) {
        self.winit_window = Some(window);
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self) -> f64 {
        self.scale_factor * self.zoom_scale_factor
    }

    /// Get the logical size of the window.
    pub fn window_size(&self) -> Size<f64> {
        Size::new(
            self.window_size.width as f64 * self.effective_scale_factor(),
            self.window_size.height as f64 * self.effective_scale_factor(),
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
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize_surface(new_size.width.max(1.0), new_size.height.max(1.0));
        }
        self.window_size = new_size;
        let size = self.window_size();
        self.render_list.borrow_mut().set_cull(Some(Rectangle::new(0.0, 0.0, size.width as f32, size.height as f32)));
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

    #[allow(clippy::too_many_arguments)]
    fn layout_window(
        &mut self,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) -> NodeId {
        let root_node = self.
            element_data
            .layout_item
            .taffy_node_id
            .expect("A root element must have a layout node.");

        let window_size = self.window_size();
        let available_space: taffy::Size<AvailableSpace> = taffy::Size {
            width: AvailableSpace::Definite(window_size.width as f32),
            height: AvailableSpace::Definite(window_size.height as f32),
        };

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            //if taffy_tree.is_layout_dirty() {
                /*let span = span!(Level::INFO, "layout(taffy)");
                let _enter = span.enter();*/
                taffy_tree.compute_layout(root_node, available_space, text_context, resource_manager.clone());
            //}

            //if self.taffy_tree.borrow().is_apply_layout_dirty() {
            /*let span = span!(Level::INFO, "layout(apply)");
                    let _enter = span.enter();*/

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
                sf
            );
            taffy_tree.apply_layout();
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
            self.scale_factor,
        );

        self.winit_window.clone().unwrap().pre_present_notify();
        self.renderer.as_mut().unwrap().sort_and_cull_render_list(render_list.borrow_mut().deref_mut());

        let window = Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.renderer.as_mut().unwrap().surface_width(),
            height: self.renderer.as_mut().unwrap().surface_height(),
        };
        self.renderer.as_mut().unwrap().prepare_render_list(self.render_list.borrow_mut().deref_mut(), resource_manager.clone(), window);

        self.renderer.as_mut().unwrap().submit(resource_manager.clone());
    }

}

impl crate::elements::core::ElementData for Window {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Window {
    fn push(&mut self, child: Rc<RefCell<dyn Element>>) -> &mut Self
    where
        Self: Sized,
    {
        let me: Weak<RefCell<dyn Element>> = self.element_data.me.clone();
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

        self
    }

    fn push_dyn(&mut self, child: Rc<RefCell<dyn Element>>) {
        self.push(child);
    }

    /// Appends multiple typed children in one call
    fn extend(&mut self, children: impl IntoIterator<Item = Rc<RefCell<dyn Element>>>) -> &mut Self
    where
        Self: Sized,
    {
        let me: Weak<RefCell<dyn Element>> = self.element_data.me.clone();
        let children: Vec<_> = children.into_iter().collect();

        for child in &children {
            child.borrow_mut().element_data_mut().parent = Some(me.clone());
        }

        self.element_data.children.extend(children.iter().cloned());

        // Add the children's taffy node.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data.layout_item.taffy_node_id.unwrap();
            for child in &children {
                if let Some(child_id) = child.borrow().element_data().layout_item.taffy_node_id {
                    taffy_tree.add_child(parent_id, child_id);
                }
            }
        });

        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for Window {
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
        if !dirty && self.element_data.scroll_state.map(|scroll_state| scroll_state.is_new()).unwrap_or_default() {
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