//! Stores one or more elements.

use crate::app::TAFFY_TREE;
use crate::elements::core::{resolve_clip_for_scrollable, ElementInternals};
use crate::elements::element_data::ElementData;
use crate::elements::{scrollable, Element};
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct Container {
    element_data: ElementData,
}

impl Container {
    pub fn new() -> Rc<RefCell<Self>> {
        let me = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), true),
            })
        });

        me.borrow_mut().element_data.create_layout_node(None);
        me
    }
}

impl crate::elements::core::ElementData for Container {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Container {
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

impl ElementInternals for Container {
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
