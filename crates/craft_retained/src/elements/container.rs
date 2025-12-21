//! Stores one or more elements.

use crate::app::{ELEMENTS, TAFFY_TREE};
use crate::elements::core::{resolve_clip_for_scrollable, ElementInternals};
use crate::elements::element_data::ElementData;
use crate::elements::{scrollable, Element};
use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::LayoutContext;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use taffy::TaffyTree;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct Container {
    element_data: ElementData,
    me: Option<Weak<RefCell<Container>>>,
}

impl Container {
    pub fn new() -> Rc<RefCell<Self>> {
        let me = Rc::new(RefCell::new(Self {
            element_data: ElementData::new(true),
            me: None,
        }));

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let node_id = taffy_tree.new_leaf(me.borrow().style().to_taffy_style()).expect("TODO: panic message");
            me.borrow_mut().element_data.layout_item.taffy_node_id = Some(node_id);
        });

        let me_element: Rc<RefCell<dyn Element>> = me.clone();

        me.borrow_mut().me = Some(Rc::downgrade(&me.clone()));
        me.borrow_mut().element_data.me = Some(Rc::downgrade(&me_element));

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(me.borrow().deref());
        });

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
        let me: Weak<RefCell<dyn Element>> = self.me.clone().unwrap() as Weak<RefCell<dyn Element>>;
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(child.clone());

        // Add the children's taffy node.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data.layout_item.taffy_node_id.unwrap();
            let child_id = child.borrow().element_data().layout_item.taffy_node_id;
            if let Some(child_id) = child_id {
                taffy_tree.add_child(parent_id, child_id).unwrap();

                taffy_tree.mark_dirty(parent_id).expect("Failed to mark taffy node dirty");
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
        let me: Weak<RefCell<dyn Element>> = self.me.clone().unwrap() as Weak<RefCell<dyn Element>>;
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
                    taffy_tree.add_child(parent_id, child_id).unwrap();
                }
            }
            taffy_tree.mark_dirty(parent_id).expect("Failed to mark taffy node dirty");
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
        taffy_tree: &mut TaffyTree<LayoutContext>,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let layout = taffy_tree.layout(self.element_data.layout_item.taffy_node_id.unwrap()).unwrap();
        self.resolve_box(position, transform, layout, z_index);
        self.apply_borders(scale_factor);

        self.element_data.apply_scroll(layout);
        self.apply_clip(clip_bounds);

        let scroll_y = self.element_data.scroll().map_or(0.0, |s| s.scroll_y() as f64);
        let child_transform = Affine::translate((0.0, -scroll_y));

        self.apply_layout_children(taffy_tree, z_index, transform * child_transform, pointer, text_context, scale_factor)
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
