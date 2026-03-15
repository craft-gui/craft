use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::{Rectangle, TrblRectangle};
use craft_renderer::RenderList;
use kurbo::{Affine, Point};

use crate::app::TAFFY_TREE;
use crate::elements::{resolve_clip_for_scrollable, ElementInternals, AsElement, Element};
use crate::elements::element_data::ElementData;
use crate::elements::{scrollable};
use crate::elements::scrollable::apply_scroll_layout;
use crate::events::{CraftMessage, Event};
use crate::layout::layout::Layout;
use crate::layout::TaffyTree;
use crate::{px, rgb, rgba};
use crate::style::{BoxShadow, Display, FlexDirection, Overflow, Position, Style};
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct Dropdown {
    pub inner: Rc<RefCell<DropdownInner>>,
}

pub struct Shape {
    pub layout: Layout,
    pub style: Box<Style>,
}

impl Shape {
    pub fn new(is_scrollable: bool) -> Self {
        let layout = Layout::new(is_scrollable);
        let style = Style::new();

        Self {
            layout,
            style
        }
    }

    pub fn create_taffy_node(&mut self) {
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let style = self.style.to_taffy_style();
            let node_id = taffy_tree.new_leaf(style);
            self.layout.taffy_node_id = Some(node_id);
        });
    }

    pub fn apply_simple_layout(&mut self, taffy_tree: &mut TaffyTree, transform: Affine, position: Point, z_index: &mut u32, scale_factor: f64, clip_bounds: Option<Rectangle>,) {
        let node = self.layout.taffy_node_id();
        let taffy_layout = taffy_tree.get_layout(node);
        self.layout.has_new_layout = taffy_tree.has_new_layout(node);
        let dirty = self.layout.is_dirty(transform, position);

        if dirty {
            self.layout.resolve_box(position, transform, taffy_layout, z_index, self.style.get_position());

            // Refactor START
            let current_style = &self.style;
            let has_border = current_style.has_border();
            let border_radius = current_style.get_border_radius();
            let border_color = &current_style.get_border_color();
            let box_shadows = current_style.get_box_shadows();
            self.layout.apply_borders(has_border, border_radius, scale_factor, border_color, box_shadows);
            // Refactor END

            // For scroll changes from taffy;
            apply_scroll_layout(&self.style, &mut self.layout, taffy_layout);
            self.layout.resolve_clip_for_scrollable(clip_bounds); // self.apply_clip
            self.layout.scroll_state.mark_old();
        }

        // For manual scroll updates.
        if !dirty && self.layout.scroll_state.is_new()
        {
            apply_scroll_layout(&self.style, &mut self.layout, taffy_layout);
            self.layout.scroll_state.mark_old();
        }

        taffy_tree.mark_seen(node);
    }
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
pub struct DropdownInner {
    element_data: ElementData,
    floating_window: Shape,
    is_floating_window_hidden: bool,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Dropdown {
    pub fn new() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<DropdownInner>>| {
            RefCell::new(DropdownInner {
                element_data: ElementData::new(me.clone(), true),
                floating_window: Shape::new(true),
                is_floating_window_hidden: true,
            })
        });

        inner.borrow_mut().element_data.create_layout_node(None);

        //inner.borrow_mut().floating_window.style.set_position(Position::Absolute);
        inner.borrow_mut().floating_window.style.set_display(Display::Flex);
        inner.borrow_mut().floating_window.style.set_flex_direction(FlexDirection::Column);

        inner.borrow_mut().floating_window.style.set_box_shadows(vec![
            BoxShadow::new(false, 0.0, 0.0, 10.0, 1.0, rgba(0, 0, 0, 30)),
            BoxShadow::new(false, 0.0, 8.0, 16.0, -2.0, rgba(0, 0, 0, 50)),
            BoxShadow::new(true, 0.0, 1.0, 1.0, 0.0, rgba(255, 255, 255, 30)),
        ]);
        let border_color = rgba(0, 0, 0, 20); // Very faint black
        inner.borrow_mut().floating_window.style.set_overflow([Overflow::Visible, Overflow::Scroll]);
        inner.borrow_mut().floating_window.style.set_border_color(TrblRectangle::new_all(border_color));
        inner.borrow_mut().floating_window.style.set_height(px(100.0));
        inner.borrow_mut().floating_window.style.set_max_height(px(100.0));
        inner.borrow_mut().floating_window.style.set_border_width(TrblRectangle::new_all(px(1)));

        inner.borrow_mut().floating_window.create_taffy_node();

        // Set the floating window's parent to the Dropdown element.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = inner.borrow_mut().element_data.layout.taffy_node_id();
            let child_id = inner.borrow_mut().floating_window.layout.taffy_node_id();
            taffy_tree.add_child(parent_id, child_id);
        });

        Self { inner }
    }
}

impl Element for Dropdown {}

impl AsElement for Dropdown {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }
}

impl crate::elements::ElementData for DropdownInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for DropdownInner {
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
        let node = self.element_data.layout.taffy_node_id();
        let layout = taffy_tree.get_layout(node);
        self.element_data.layout.has_new_layout = taffy_tree.has_new_layout(node);
        let dirty = self.element_data.layout.is_dirty(transform, position);

        if dirty {
            self.resolve_box(position, transform, layout, z_index);
            self.apply_borders(scale_factor);
            self.apply_clip(clip_bounds);
            self.element_data.layout.scroll_state.mark_old();
        }
        taffy_tree.mark_seen(node);


        // Position the floating window below the dropdown with a gap.
        let floating_window_position = Point::new(
            self.element_data.layout.computed_box.position.x,
            self.element_data.layout.computed_box.position.y + self.element_data.layout.computed_box.size.height as f64
        );

        self.floating_window.apply_simple_layout(taffy_tree, transform, floating_window_position, z_index, scale_factor, None /*clip_bounds*/);
        let scroll_y = self.floating_window.layout.scroll_state.scroll_y() as f64;
        let child_transform = Affine::translate((0.0, -scroll_y));

        for child in &self.element_data.children {
            child.borrow_mut().apply_layout(
                taffy_tree,
                floating_window_position,
                z_index,
                transform * child_transform,
                pointer,
                text_context,
                self.floating_window.layout.clip_bounds,
                scale_factor,
            );
        }
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout.computed_box_transformed.border_rectangle();
        let floating_window_rect = self.floating_window.layout.computed_box_transformed.border_rectangle();

        if floating_window_rect.contains(&point) {
            return true;
        }

        if let Some(clip) = element_data.layout.clip_bounds {
            let rect_intersection = match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            };

            rect_intersection
        } else {
            rect.contains(&point)
        }
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

        if true || !self.is_floating_window_hidden {
            renderer.start_overlay();
            self.draw_borders(renderer, scale_factor);

            let current_style = self.floating_window.style.as_ref();
            self.floating_window.layout.draw_borders(renderer, current_style, scale_factor);

            self.maybe_start_layer(renderer, scale_factor);
            self.draw_children(renderer, text_context, pointer, scale_factor);
            self.maybe_end_layer(renderer);
            renderer.end_overlay();
        }
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        let mut element_data = self.element_data;
        scrollable::on_scroll_events(
            self,
            &element_data.style,
            &mut element_data.layout,
            message,
            event
        );

        let pb = match message {
            CraftMessage::PointerButtonUp(pb) => Some(pb),
            _ => None
        };

        // if self.is_floating_window_hidden {
        //     return;
        // }

        let floating_window_rect = self.floating_window.layout.computed_box_transformed.border_rectangle();

        //let scroll_result = scrollable::on_scroll_events_no_pointer_advance(&self.floating_window.style, &mut self.floating_window.layout, message, event);
        //scrollable::on_scroll_events_handle_advance_result(self, scroll_result);


        if let Some(pb) = pb {
            self.is_floating_window_hidden = !self.is_floating_window_hidden;
        }


    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        let me: Weak<RefCell<dyn ElementInternals>> = self.element_data.me.clone();
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(child.clone());

        // Add the children to the floating window layout.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.floating_window.layout.taffy_node_id.unwrap();
            let child_id = child.borrow().element_data().layout.taffy_node_id();
            taffy_tree.add_child(parent_id, child_id);
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
