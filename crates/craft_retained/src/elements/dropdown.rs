use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::{Rectangle, TrblRectangle};
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use peniko::Color;
use ui_events::pointer::PointerId;
use crate::app::{request_apply_layout, CURRENT_WINDOW_ID, TAFFY_TREE, WINDOW_MANAGER};
use crate::elements::{resolve_clip_for_scrollable, ElementInternals, AsElement, Element, ElementData, Window};
use crate::elements::element_data::ElementData as ElementDataStruct;
use crate::elements::{scrollable};
use crate::elements::scrollable::{apply_scroll_layout, draw_scrollbar, handle_scroll_logic_advance};
use crate::events::{CraftMessage, Event};
use crate::layout::layout::Layout;
use crate::layout::TaffyTree;
use crate::{px, rgb, rgba, CraftError};
use crate::document::DocumentManager;
use crate::elements::traits::DeepClone;
use crate::style::{AlignItems, BoxShadow, Display, FlexDirection, JustifyContent, Overflow, Position, Style, Unit};
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct Dropdown {
    pub inner: Rc<RefCell<DropdownInner>>,
}

#[derive(Clone)]
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

    pub fn apply_simple_layout(&mut self, taffy_tree: &mut TaffyTree, transform: Affine, position: Point, z_index: &mut u32, scale_factor: f64, clip_bounds: Option<Rectangle>) {
        let node = self.layout.taffy_node_id();
        let taffy_layout = taffy_tree.get_layout(node);
        self.layout.has_new_layout = taffy_tree.has_new_layout(node);

        let dirty = self.layout.has_new_layout
            || transform != self.layout.get_transform()
            || position != self.layout.position;

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
#[derive(Clone)]
pub struct DropdownInner {
    element_data: ElementDataStruct,
    floating_window: Shape,
    is_floating_window_hidden: bool,
    selected_element: Option<Rc<RefCell<dyn ElementInternals>>>,
    selected_element_index: Option<usize>,
    selected_bg_color: Option<Color>,
    hovered_bg_color: Option<Color>,
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
                element_data: ElementDataStruct::new(me.clone(), true),
                floating_window: Shape::new(true),
                is_floating_window_hidden: true,
                selected_element: None,
                selected_element_index: None,
                selected_bg_color: Some(Color::from_rgba8(109, 113, 228, 100)),
                hovered_bg_color: Some(Color::from_rgba8(100, 100, 100, 100)),
            })
        });

        inner.borrow_mut().element_data.create_layout_node(None);
        inner.borrow_mut().element_data.style.set_display(Display::Flex);
        inner.borrow_mut().element_data.style.set_justify_content(Some(JustifyContent::Center));
        inner.borrow_mut().element_data.style.set_align_items(Some(AlignItems::Center));

        inner.borrow_mut().floating_window.style.set_position(Position::Absolute);
        inner.borrow_mut().floating_window.style.set_display(Display::Flex);
        inner.borrow_mut().floating_window.style.set_flex_direction(FlexDirection::Column);

        inner.borrow_mut().floating_window.style.set_box_shadows(vec![
            BoxShadow::new(false, 0.0, 4.0, 16.0, 0.0, rgba(0, 0, 0, 140)), // 0.55
            BoxShadow::new(false, 0.0, 20.0, 60.0, 0.0, rgba(0, 0, 0, 90)),  // 0.35
        ]);

        let border_color = rgba(0, 0, 0, 64);
        inner.borrow_mut().floating_window.style.set_padding(TrblRectangle::new_all(px(2.0)));
        inner.borrow_mut().floating_window.style.set_width(Unit::Percentage(100.0));
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
    fn element_data(&self) -> &ElementDataStruct {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementDataStruct {
        &mut self.element_data
    }
}

impl DropdownInner {
    fn set_selected_element(&mut self, child_index: usize) {
        // Remove the old selected element from the layout tree.
        if let Some(old_selected_element) = &self.selected_element {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.remove_subtree(old_selected_element.borrow().element_data().layout.taffy_node_id());
            });
        }

        let child = self.element_data.children.get(child_index).expect("There is no child at this index.");
        self.selected_element = Some(child.clone().borrow().deep_clone());
        self.selected_element_index = Some(child_index);
        let selected_element_id = self.selected_element.as_ref().unwrap().borrow().element_data().layout.taffy_node_id();

        // Add the selected element to the parent's layout tree at index 1.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data.layout.taffy_node_id.unwrap();
            taffy_tree.add_child_at_index(parent_id, selected_element_id, 1);
        });

    }
}

impl ElementInternals for DropdownInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

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
        let dirty = self.element_data.layout.has_new_layout
            || transform != self.element_data.layout.get_transform()
            || position != self.element_data.layout.position;

        if dirty {
            self.resolve_box(position, transform, layout, z_index);
            self.apply_borders(scale_factor);
            self.apply_clip(clip_bounds);
            self.element_data.layout.scroll_state.mark_old();
        }
        taffy_tree.mark_seen(node);

        if let Some(selected_element) = &self.selected_element {
            selected_element.borrow_mut().apply_layout(
                taffy_tree,
                self.element_data().layout.computed_box.position,
                z_index,
                transform,
                pointer,
                text_context,
                self.element_data().layout.clip_bounds,
                scale_factor,
            );
            taffy_tree.mark_seen(selected_element.borrow().element_data().layout.taffy_node_id());
        }

        // Position the floating window below the dropdown with a gap.
        let floating_window_position = Point::new(
            self.element_data.layout.computed_box.position.x,
            self.element_data.layout.computed_box.position.y + self.element_data.layout.computed_box.size.height as f64
        );


        self.floating_window.apply_simple_layout(taffy_tree, transform, floating_window_position, z_index, scale_factor, None);


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

        if let Some(selected_element) = &self.selected_element {
            let mut binding = selected_element.borrow_mut();
            binding.draw(renderer, text_context, pointer, scale_factor);
        }

        if !self.is_floating_window_hidden {
            //renderer.start_overlay();

            let current_style = self.floating_window.style.as_ref();
            self.floating_window.layout.draw_borders(renderer, current_style, scale_factor);

            renderer.push_layer(self.floating_window.layout.computed_box_transformed.padding_rectangle().scale(scale_factor));

            for (index, child) in self.children().iter().enumerate() {
                let mut child_rect = child.borrow_mut().element_data().layout.computed_box_transformed.border_rectangle();
                child_rect.x = self.floating_window.layout.computed_box_transformed.position.x as f32;
                child_rect.width = self.floating_window.layout.computed_box_transformed.size.width as f32;



                if let Some(selected_index) = self.selected_element_index && index == selected_index {
                    renderer.draw_rect(child_rect, self.selected_bg_color.unwrap());
                } else if child.borrow_mut().element_data().is_hovered {
                    renderer.draw_rect(child_rect, self.hovered_bg_color.unwrap());
                }

                child.borrow_mut().draw(renderer, text_context, pointer, scale_factor);
            }

            renderer.pop_layer();

            draw_scrollbar(&self.floating_window.style, &self.floating_window.layout, renderer, scale_factor);
           //renderer.end_overlay();
        }
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        if self.is_floating_window_hidden {
            self.release_pointer_capture(PointerId::new(1).unwrap());
        }

        let pb = match message {
            CraftMessage::PointerButtonUp(pb) => Some(pb),
            _ => None
        };

        if let Some(pb) = pb {
            // TODO Should pb.state.position be in logical coordinates?
            let pointer_position = Point::new(pb.state.position.x, pb.state.position.y);

            if self.element_data.layout.computed_box_transformed.border_rectangle().contains(&pointer_position) {
                self.is_floating_window_hidden = !self.is_floating_window_hidden;
            }

            if !self.floating_window.layout.computed_box_transformed.border_rectangle().contains(&pointer_position) || self.is_floating_window_hidden {
                return;
            }

            for (child_index, child) in self.children().iter().map(|r| r.clone()).enumerate() {
                let contains = child.borrow().element_data().layout.computed_box_transformed.border_rectangle().contains(&pointer_position);

                if contains {
                    self.is_floating_window_hidden = !self.is_floating_window_hidden.clone();
                    self.set_selected_element(child_index);
                    break;
                }
            }
        }

        let floating_window = &mut self.floating_window;
        let result = handle_scroll_logic_advance(&*floating_window.style, &mut floating_window.layout, message, event);

        if result.request_apply_layout {
            request_apply_layout(self.element_data.layout.taffy_node_id.unwrap());
        }

        if result.set_pointer_capture {
            self.set_pointer_capture(PointerId::new(1).unwrap())
        } else if result.release_pointer_capture {
            self.release_pointer_capture(PointerId::new(1).unwrap());
        }
        // ---


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
