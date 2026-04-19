//! An element to select a single item from a collapsable vertical list of options.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::{BezPath, Rectangle, TrblRectangle};

use craft_renderer::{Brush, RenderList};

use craft_primitives::geometry::{Affine, Point, Vec2};

use peniko::Color;

use ui_events::pointer::PointerId;

use crate::app::{TAFFY_TREE, queue_event, request_apply_layout};
use crate::elements::element_data::ElementData as ElementDataStruct;
use crate::elements::scrollable::{apply_scroll_layout, draw_scrollbar, handle_scroll_logic_advance};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementData, ElementInternals, resolve_clip_for_scrollable};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::layout::layout::Layout;
use crate::style::{AlignItems, BoxShadow, Display, FlexDirection, JustifyContent, Overflow, Position, Style, Unit};
use crate::text::text_context::TextContext;
use crate::{auto, px, rgba};

/// An element to select a single item from a collapsable vertical list of options.
///
/// # Example
///
/// ```no_run
/// use craft_retained::elements::{Dropdown, Element, Text, Window};
/// use craft_retained::{CraftOptions, craft_main, px};
///
/// fn main() {
///     Window::new("Dropdown").push(
///         Dropdown::new()
///             .width(px(100))
///             .push(Text::new("Item 1").font_size(20.0).selectable(false))
///             .push(Text::new("Item 2").font_size(20.0).selectable(false))
///             .push(Text::new("Item 3").font_size(20.0).selectable(false))
///             .selected_item(0),
///     );
///     craft_main(CraftOptions::basic("Dropdown"));
/// }
/// ```
#[derive(Clone)]
pub struct Dropdown {
    pub inner: Rc<RefCell<DropdownInner>>,
}

#[derive(Clone)]
pub struct Shape {
    pub layout: Layout,
    pub style: Box<Style>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct DropdownInner {
    element_data: ElementDataStruct,
    floating_window: Shape,
    arrow: Shape,
    is_floating_window_hidden: bool,
    selected_element: Option<Rc<RefCell<dyn ElementInternals>>>,
    selected_element_index: Option<usize>,
    currently_hovered_element: Option<usize>,
    hovered_bg_color: Option<Color>,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Dropdown {}

impl Drop for DropdownInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Dropdown {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }
}

impl ElementData for DropdownInner {
    fn element_data(&self) -> &ElementDataStruct {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementDataStruct {
        &mut self.element_data
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
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.taffy_node_id();
        let layout = taffy_tree.get_layout(node);
        self.element_data.layout.has_new_layout = taffy_tree.has_new_layout(node);
        let dirty = self.element_data.layout.has_new_layout
            || transform != self.element_data.layout.get_transform()
            || position != self.element_data.layout.position
            || clip_bounds != self.element_data.layout.parent_clip;

        if dirty {
            self.resolve_box(position, transform, layout, z_index);
            self.apply_borders(scale_factor);
            self.apply_clip(clip_bounds);
            self.element_data.layout.parent_clip = clip_bounds;
            self.element_data.layout.scroll_state.mark_old();
        }
        taffy_tree.mark_seen(node);

        if let Some(selected_element) = &self.selected_element {
            selected_element.borrow_mut().apply_layout(
                taffy_tree,
                self.element_data().layout.computed_box.position,
                z_index,
                transform,
                text_context,
                self.element_data().layout.clip_bounds,
                scale_factor,
            );
            taffy_tree.mark_seen(selected_element.borrow().element_data().layout.taffy_node_id());
        }

        // Position the floating window below the dropdown with a gap.
        let floating_window_position = Point::new(
            self.element_data.layout.computed_box.position.x,
            self.element_data.layout.computed_box.position.y + self.element_data.layout.computed_box.size.height as f64,
        );

        self.floating_window.apply_simple_layout(
            taffy_tree,
            transform,
            floating_window_position,
            z_index,
            scale_factor,
            None,
        );
        self.arrow.apply_simple_layout(
            taffy_tree,
            transform,
            self.element_data().layout.computed_box.position,
            z_index,
            scale_factor,
            self.element_data().layout.clip_bounds,
        );

        let scroll_y = self.floating_window.layout.scroll_state.scroll_y() as f64;
        let child_transform = Affine::translate((0.0, -scroll_y));

        for child in &self.element_data.children {
            child.borrow_mut().apply_layout(
                taffy_tree,
                floating_window_position,
                z_index,
                transform * child_transform,
                text_context,
                self.floating_window.layout.clip_bounds,
                scale_factor,
            );
        }
    }

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        if !self.is_visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);
        if self.is_floating_window_hidden {
            self.add_hit_testable(renderer, true, scale_factor);
        }

        self.draw_selected_element(renderer, text_context, scale_factor);

        // Draw the arrow
        let arrow_rect = self.arrow.layout.computed_box_transformed.border_rectangle();
        let thickness = 2.0 * scale_factor;
        let mut path = BezPath::new();
        let left_x = arrow_rect.x as f64;
        let right_x = (arrow_rect.x + arrow_rect.width) as f64;
        let center_x = (arrow_rect.x + arrow_rect.width / 2.0) as f64;
        let top_y = arrow_rect.y as f64;
        let bottom_y = (arrow_rect.y + arrow_rect.height) as f64;
        path.move_to(Point::new(left_x, top_y));
        path.line_to(Point::new(center_x, bottom_y));
        path.line_to(Point::new(right_x, top_y));
        path.line_to(Point::new(right_x - thickness, top_y));
        path.line_to(Point::new(center_x, bottom_y - thickness));
        path.line_to(Point::new(left_x + thickness, top_y));
        path.close_path();
        path.apply_affine((Affine::IDENTITY).then_translate(Vec2::new(0.0, arrow_rect.height as f64 / 4.0)));
        let arrow_color = Color::from_rgba8(75, 75, 77, 255);
        renderer.fill_bez_path(path, Brush::Color(arrow_color));

        if !self.is_floating_window_hidden {
            renderer.start_overlay();
            // If the dropdown menu is open, then we must add a hit testable after we start
            // an overlay, so that it is properly sorted in the event target selection phase.
            self.add_hit_testable(renderer, true, scale_factor);

            let current_style = self.floating_window.style.as_ref();
            self.floating_window
                .layout
                .draw_borders(renderer, current_style, scale_factor);

            renderer.push_layer(
                self.floating_window
                    .layout
                    .computed_box_transformed
                    .padding_rectangle()
                    .scale(scale_factor),
            );

            self.draw_children(renderer, text_context, scale_factor);

            renderer.pop_layer();

            draw_scrollbar(
                &self.floating_window.style,
                &self.floating_window.layout,
                renderer,
                scale_factor,
            );
            renderer.end_overlay();
        }
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        // Take focus if clicked.
        if let EventKind::PointerButtonDown(_pb) = message {
            self.focus();
        }

        let list_layout = &self.floating_window.layout;
        let list_box = list_layout.computed_box_transformed.border_rectangle();
        let list_scroll_box = list_layout.computed_scroll_track;

        self.update_most_recently_hovered_child(message, list_box, list_scroll_box);

        if let EventKind::PointerButtonUp(pb) = message {
            // TODO Should pb.state.position be in logical coordinates?
            let pointer_position = Point::new(pb.state.position.x, pb.state.position.y);
            let is_pointer_in_select_box = self
                .element_data
                .layout
                .computed_box_transformed
                .border_rectangle()
                .contains(&pointer_position);
            let is_pointer_in_window = self
                .floating_window
                .layout
                .computed_box_transformed
                .border_rectangle()
                .contains(&pointer_position);
            let is_pointer_in_scrollbar = self
                .floating_window
                .layout
                .computed_scroll_track
                .contains(&pointer_position);

            self.handle_click_outside_menu(is_pointer_in_select_box, is_pointer_in_window);
            self.handle_click_in_select_box(is_pointer_in_select_box);
            self.handle_child_click(event, &pointer_position, is_pointer_in_window, is_pointer_in_scrollbar);
        }

        // Handle updating the scroll state.
        // TODO: The dropdown scroll logic needs refactoring.
        let floating_window = &mut self.floating_window;
        let result = handle_scroll_logic_advance(&floating_window.style, &mut floating_window.layout, message, event);
        if result.request_apply_layout {
            request_apply_layout(self.element_data.layout.taffy_node_id.unwrap());
        }
        if result.set_pointer_capture {
            self.set_pointer_capture(PointerId::new(1).unwrap())
        } else if result.release_pointer_capture {
            self.release_pointer_capture(PointerId::new(1).unwrap());
        }
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        let me: Weak<RefCell<dyn ElementInternals>> = self.element_data.me.clone();
        let me_window = self.element_data.window.clone();
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(child.clone());
        child.borrow_mut().element_data_mut().window = me_window;
        child.borrow_mut().propagate_window_down();

        // Add the children to the floating window layout.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.floating_window.layout.taffy_node_id.unwrap();
            let child_id = child.borrow().element_data().layout.taffy_node_id();
            taffy_tree.add_child(parent_id, child_id);
        });
    }

    fn draw_children(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        for (index, child) in self.children().iter().enumerate() {
            let floating_window_box = &self.floating_window.layout.computed_box_transformed;
            let mut child_rect = child
                .borrow_mut()
                .element_data()
                .layout
                .computed_box_transformed
                .border_rectangle();

            child_rect.x = floating_window_box.position.x as f32;
            child_rect.width = floating_window_box.size.width;

            let is_hovered = self.currently_hovered_element == Some(index);
            if is_hovered {
                renderer.draw_rect(child_rect, self.hovered_bg_color.unwrap());
            }

            child.borrow_mut().draw(renderer, text_context, scale_factor);
        }
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout.computed_box_transformed.border_rectangle();
        //let floating_window_rect = self.floating_window.layout.computed_box_transformed.border_rectangle();

        if !self.is_floating_window_hidden {
            return true;
        }

        if let Some(clip) = element_data.layout.clip_bounds {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            rect.contains(&point)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Shape {
    pub fn new(is_scrollable: bool) -> Self {
        let layout = Layout::new(is_scrollable);
        let style = Style::new();

        Self { layout, style }
    }

    pub fn create_taffy_node(&mut self) {
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let style = self.style.to_taffy_style();
            let node_id = taffy_tree.new_leaf(style);
            self.layout.taffy_node_id = Some(node_id);
        });
    }

    pub fn apply_simple_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        transform: Affine,
        position: Point,
        z_index: &mut u32,
        scale_factor: f64,
        clip_bounds: Option<Rectangle>,
    ) {
        let node = self.layout.taffy_node_id();
        let taffy_layout = taffy_tree.get_layout(node);
        self.layout.has_new_layout = taffy_tree.has_new_layout(node);

        let dirty =
            self.layout.has_new_layout || transform != self.layout.get_transform() || position != self.layout.position;

        if dirty {
            self.layout
                .resolve_box(position, transform, taffy_layout, z_index, self.style.get_position());

            // Refactor START
            let current_style = &self.style;
            let has_border = current_style.has_border();
            let border_radius = current_style.get_border_radius();
            let border_color = &current_style.get_border_color();
            let box_shadows = current_style.get_box_shadows();
            self.layout
                .apply_borders(has_border, border_radius, scale_factor, border_color, box_shadows);
            // Refactor END

            // For scroll changes from taffy;
            apply_scroll_layout(&self.style, &mut self.layout, taffy_layout);
            self.layout.resolve_clip_for_scrollable(clip_bounds); // self.apply_clip
            self.layout.scroll_state.mark_old();
        }

        // For manual scroll updates.
        if !dirty && self.layout.scroll_state.is_new() {
            apply_scroll_layout(&self.style, &mut self.layout, taffy_layout);
            self.layout.scroll_state.mark_old();
        }

        taffy_tree.mark_seen(node);
    }
}

impl Dropdown {
    pub fn new() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<DropdownInner>>| {
            RefCell::new(DropdownInner {
                element_data: ElementDataStruct::new(me.clone(), true),
                floating_window: Shape::new(true),
                arrow: Shape::new(false),
                is_floating_window_hidden: true,
                selected_element: None,
                selected_element_index: None,
                currently_hovered_element: None,
                hovered_bg_color: Some(Color::from_rgba8(213, 213, 215, 255)),
            })
        });

        let border_color = rgba(0, 0, 0, 64);
        let border_width = px(1.0);
        let border_radius = [(5.0, 5.0); 4];

        inner.borrow_mut().element_data.create_layout_node(None);
        inner.borrow_mut().element_data.style.set_display(Display::Flex);
   
        inner
            .borrow_mut()
            .element_data
            .style
            .set_align_items(Some(AlignItems::Center));
        inner
            .borrow_mut()
            .element_data
            .style
            .set_padding(TrblRectangle::new(px(2.5), px(0.0), px(2.5), px(6.0)));
        inner
            .borrow_mut()
            .element_data
            .style
            .set_border_width(TrblRectangle::new_all(border_width));
        inner.borrow_mut().element_data.style.set_border_radius(border_radius);
        inner
            .borrow_mut()
            .element_data
            .style
            .set_border_color(TrblRectangle::new_all(border_color));

        inner
            .borrow_mut()
            .floating_window
            .style
            .set_background_color(Color::WHITE);
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_position(Position::Absolute);
        inner.borrow_mut().floating_window.style.set_display(Display::Flex);
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_flex_direction(FlexDirection::Column);
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_box_shadows(vec![BoxShadow::new(false, 0.0, 4.0, 8.0, 1.0, rgba(0, 0, 0, 255))]);
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_padding(TrblRectangle::new(px(2.5), px(0.0), px(2.5), px(6.0)));
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_width(Unit::Percentage(100.0));
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_overflow([Overflow::Visible, Overflow::Scroll]);
        inner.borrow_mut().floating_window.style.set_height(px(120.0));
        inner.borrow_mut().floating_window.style.set_max_height(px(100.0));
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_border_width(TrblRectangle::new_all(border_width));
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_border_radius(border_radius);
        inner
            .borrow_mut()
            .floating_window
            .style
            .set_border_color(TrblRectangle::new_all(border_color));
        inner.borrow_mut().floating_window.create_taffy_node();

        inner.borrow_mut().arrow.style.set_width(px(12.0));
        inner.borrow_mut().arrow.style.set_height(px(6.0));
        inner
            .borrow_mut()
            .arrow
            .style
            .set_margin(TrblRectangle::new(px(0.0), px(8.0), px(0.0), auto()));
        inner.borrow_mut().arrow.create_taffy_node();

        // Set the floating window's parent and the arrow to the Dropdown element.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = inner.borrow_mut().element_data.layout.taffy_node_id();
            let floating_window_child_id = inner.borrow_mut().floating_window.layout.taffy_node_id();
            let arrow_child_id = inner.borrow_mut().arrow.layout.taffy_node_id();
            taffy_tree.add_child(parent_id, floating_window_child_id);
            taffy_tree.add_child(parent_id, arrow_child_id);
        });

        Self { inner }
    }

    pub fn selected_item(self, index: usize) -> Self {
        let binding = self.inner.clone();
        let mut inner = binding.borrow_mut();
        inner.set_selected_element(index);
        self
    }

    pub fn get_selected_item(self) -> usize {
        self.inner.borrow().selected_element_index.unwrap()
    }
}

impl DropdownInner {
    fn draw_selected_element(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        if let Some(selected_element) = &self.selected_element {
            let mut binding = selected_element.borrow_mut();
            binding.draw(renderer, text_context, scale_factor);
        }
    }

    fn set_selected_element(&mut self, child_index: usize) {
        // Remove the old selected element from the layout tree.
        if let Some(old_selected_element) = &self.selected_element {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.unparent_node(old_selected_element.borrow().element_data().layout.taffy_node_id());
            });
        }

        let child = self
            .element_data
            .children
            .get(child_index)
            .expect("There is no child at this index.");
        self.selected_element = Some(child.clone().borrow().deep_clone());
        self.selected_element_index = Some(child_index);
        let selected_element_id = self
            .selected_element
            .as_ref()
            .unwrap()
            .borrow()
            .element_data()
            .layout
            .taffy_node_id();

        // Add the selected element to the parent's layout tree at index 1.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data.layout.taffy_node_id.unwrap();
            taffy_tree.add_child_at_index(parent_id, selected_element_id, 1);
        });
    }

    fn update_most_recently_hovered_child(
        &mut self,
        message: &EventKind,
        list_box: Rectangle,
        list_scroll_box: Rectangle,
    ) {
        if let EventKind::PointerMovedEvent(pb) = message {
            let pointer_position = Point::new(pb.current.position.x, pb.current.position.y);
            let is_pointer_in_list = list_box.contains(&pointer_position);
            let is_pointer_in_scrollbar = list_scroll_box.contains(&pointer_position);

            if is_pointer_in_list && !is_pointer_in_scrollbar {
                let hovered_child = self.children().iter().enumerate().find_map(|(index, child)| {
                    let contains = child
                        .borrow()
                        .element_data()
                        .layout
                        .computed_box_transformed
                        .border_rectangle()
                        .contains(&pointer_position);

                    if contains {
                        return Some(index);
                    }

                    None
                });

                if let Some(hovered_child) = hovered_child {
                    self.currently_hovered_element = Some(hovered_child);
                }
            }
        }
    }

    fn handle_click_in_select_box(&mut self, is_pointer_in_select_box: bool) {
        if is_pointer_in_select_box {
            self.is_floating_window_hidden = !self.is_floating_window_hidden;
            self.currently_hovered_element = self.selected_element_index;

            if self.is_floating_window_hidden {
                self.release_pointer_capture(PointerId::new(1).unwrap());
            }
        }
    }

    fn handle_click_outside_menu(&mut self, is_pointer_in_select_box: bool, is_pointer_in_window: bool) {
        if !self.is_floating_window_hidden && !is_pointer_in_window && !is_pointer_in_select_box {
            self.is_floating_window_hidden = true;
        }
    }

    fn handle_child_click(
        &mut self,
        event: &mut Event,
        pointer_position: &Point,
        is_pointer_in_window: bool,
        is_pointer_in_scrollbar: bool,
    ) {
        if is_pointer_in_window && !is_pointer_in_scrollbar {
            let mut should_hide_window = false;
            for (child_index, child) in self.children().iter().cloned().enumerate() {
                let contains = child
                    .borrow()
                    .element_data()
                    .layout
                    .computed_box_transformed
                    .border_rectangle()
                    .contains(pointer_position);

                if contains {
                    should_hide_window = true;
                    self.set_selected_element(child_index);
                    self.release_pointer_capture(PointerId::new(1).unwrap());

                    let new_event = Event::new(event.target.clone());
                    queue_event(new_event, EventKind::DropdownItemSelected(child_index));

                    break;
                }
            }

            if should_hide_window {
                self.is_floating_window_hidden = true;
            }
        }
    }
}
