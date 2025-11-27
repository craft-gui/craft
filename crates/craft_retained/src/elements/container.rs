//! Stores one or more elements.

use std::any::Any;
use crate::elements::core::ElementData as ElementDataTrait;
use crate::elements::core::{resolve_clip_for_scrollable, ElementInternals};
use crate::elements::element_data::ElementData;
use crate::elements::{scrollable, Element};
use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::LayoutContext;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::app::TAFFY_TREE;

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

        me.borrow_mut().me = Some(Rc::downgrade(&me.clone()));
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
    fn push(&mut self, child: Rc<RefCell<dyn Element>>) -> &mut Self where Self: Sized {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for Container {
    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, scale_factor: f64) -> Option<NodeId> {
        for child in &mut self.element_data.children {
            child.borrow_mut().compute_layout(taffy_tree, scale_factor);
        }

        if self.element_data.style.is_dirty {
            let node_id = self.element_data.layout_item.taffy_node_id.unwrap();
            let style: taffy::Style = self.element_data.style.to_taffy_style();
            taffy_tree.set_style(node_id, style).expect("Failed to set style on node.");
            self.element_data.style.is_dirty = false;
        }

        self.element_data.layout_item.taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let layout = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, layout, z_index);
        self.finalize_borders();

        self.element_data.finalize_scroll(layout);
        self.resolve_clip(clip_bounds);

        let scroll_y = self.element_data.scroll().map_or(0.0, |s| s.scroll_y() as f64);
        let child_transform = Affine::translate((0.0, -scroll_y));

        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.borrow().element_data().layout_item.taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.borrow_mut().finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform * child_transform,
                pointer,
                text_context,
                self.element_data.layout_item.clip_bounds,
            );
        }
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        let current_style = self.element_data.current_style();

        if !current_style.visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        for child in self.children() {
            child.borrow_mut().draw(renderer, text_context, pointer, window.clone(), scale_factor);
        }
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
        //self.on_style_event(message, should_style, event);
        //self.maybe_unset_focus(message, event, target);

        scrollable::on_scroll_events(self, message, event);
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }
}
