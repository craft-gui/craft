//! An element to select a single item from a collapsable vertical list of options.

use retgui_primitives::geometry::{BezPath, Rectangle, TrblRectangle};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use retgui_renderer::Brush;

use retgui_primitives::geometry::{Affine, Point, Vec2};

use peniko::Color;

use winit::keyboard::KeyCode;

use crate::app::{GUMMY_TREE, queue_event};
use crate::elements::element_data::ElementData as ElementDataStruct;
use crate::elements::scrollable::{apply_scroll_layout, draw_scrollbar, handle_scroll_logic_advance, set_scroll_y};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, DynElement, Element, ElementData, ElementInternals};
use crate::events::{DropdownItemSelectedEvent, DropdownToggledEvent, Event, EventKind, PointerButton, PointerId};
use crate::layout::GummyTree;
use crate::layout::layout::Layout;
use crate::style::{AlignItems, BoxShadow, Display, FlexDirection, Overflow, Position, Style, Unit};
use crate::text::text_context::TextContext;
use crate::{auto, px, rgba};
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

/// An element to select a single item from a collapsable vertical list of options.
///
/// # Example
///
/// ```no_run
/// use retgui::elements::{Dropdown, Element, Text, Window};
/// use retgui::{RetGuiOptions, retgui_main, px};
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
///     retgui_main(RetGuiOptions::basic("Dropdown"));
/// }
/// ```
#[derive(Clone)]
pub struct Dropdown {
    pub(crate) inner: Rc<RefCell<DropdownInner>>,
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
pub(crate) struct DropdownInner {
    element_data: ElementDataStruct,
    floating_window: Shape,
    arrow: Shape,
    is_floating_window_hidden: bool,
    selected_element: Option<Rc<RefCell<dyn ElementInternals>>>,
    selected_element_index: Option<usize>,
    currently_hovered_element: Option<usize>,
    hovered_bg_brush: Option<Brush>,
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
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
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
    fn deep_clone(&self) -> DynElement {
        let element = clone_element::<Self, _>(self, |element, gummy_tree| {
            let mut element = element.borrow_mut();
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me.clone();
            let parent = element.element_data.layout.gummy_node_id();
            let floating_window_node = gummy_tree.clone_node(element.floating_window.layout.gummy_node_id());
            let arrow_node = gummy_tree.clone_node(element.arrow.layout.gummy_node_id());

            element.floating_window.layout.gummy_node_id = Some(floating_window_node);
            element.arrow.layout.gummy_node_id = Some(arrow_node);
            element.selected_element = None;

            gummy_tree.add_child(parent, floating_window_node);
            gummy_tree.add_child(parent, arrow_node);
            gummy_tree.register_owner(floating_window_node, owner_id, owner.clone());
            gummy_tree.register_owner(arrow_node, owner_id, owner);
            Some(floating_window_node)
        });
        let selected_element_index = element.borrow().selected_element_index;
        if let Some(index) = selected_element_index {
            element.borrow_mut().set_selected_element(index);
        }
        DynElement::new(element)
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.element_data.applied_scale_factor = scale_factor;
        self.apply_borders(scale_factor);
        self.floating_window.apply_borders(scale_factor);
        self.arrow.apply_borders(scale_factor);
        for child in &self.element_data.children {
            child.inner.borrow_mut().set_scale_factor(scale_factor);
        }
        if let Some(selected_element) = &self.selected_element {
            selected_element.borrow_mut().set_scale_factor(scale_factor);
        }
        self.mark_dirty();
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.gummy_node_id();
        let layout = gummy_tree.get_layout(node);
        self.element_data.layout.has_new_layout = gummy_tree.has_new_layout(node);
        let has_new_layout = self.element_data.layout.has_new_layout;

        if has_new_layout {
            self.resolve_box(layout, z_index);
            self.apply_borders(scale_factor);
        }
        gummy_tree.mark_seen(node);

        self.floating_window
            .apply_simple_layout(gummy_tree, z_index, scale_factor);
        self.arrow.apply_simple_layout(gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.maybe_start_overlay(renderer);

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);
        if self.is_floating_window_hidden {
            self.add_hit_testable(renderer, true, scale_factor);
        }

        self.draw_selected_element(renderer, resource_manager.clone(), text_context, scale_factor);

        // Draw the arrow
        let arrow_rect = self
            .arrow
            .layout
            .local_box_in_parent()
            .border_rectangle()
            .scale(scale_factor);
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

            let dropdown_transform = renderer.get_transform();
            let floating_offset = Affine::translate((
                0.0,
                self.element_data.layout.computed_box.size.height as f64 * scale_factor,
            ));
            renderer.set_transform(
                dropdown_transform * floating_offset * self.floating_window.layout.local_transform(scale_factor),
            );
            let render_transform = renderer.get_transform();
            let logical_transform = Affine::scale(1.0 / scale_factor) * render_transform * Affine::scale(scale_factor);
            let logical_clip = renderer.get_clip().map(|clip| clip.scale(1.0 / scale_factor));
            self.floating_window
                .layout
                .update_render_state(logical_transform, logical_clip);

            let current_style = self.floating_window.style.as_ref();
            self.floating_window
                .layout
                .draw_borders(renderer, current_style, scale_factor);

            renderer.push_layer(
                self.floating_window
                    .layout
                    .local_box()
                    .padding_rectangle()
                    .scale(scale_factor),
            );

            self.draw_children(renderer, resource_manager.clone(), scale_factor, text_context);

            renderer.pop_layer();

            draw_scrollbar(
                &self.floating_window.style,
                &self.floating_window.layout,
                renderer,
                scale_factor,
            );
            renderer.set_transform(dropdown_transform);
            renderer.end_overlay();
        }

        self.maybe_end_overlay(renderer);
    }

    fn add_hit_testable(&mut self, renderer: &mut dyn Renderer, hit_testable: bool, scale_factor: f64) {
        if !hit_testable {
            return;
        }

        let bounds = if self.is_floating_window_hidden {
            self.element_data
                .layout
                .local_box()
                .border_rectangle()
                .scale(scale_factor)
        } else if let Some(cull) = renderer.render_list().cull {
            cull.apply_transform(renderer.get_transform().inverse())
        } else {
            self.element_data
                .layout
                .local_box()
                .border_rectangle()
                .scale(scale_factor)
        };
        renderer.push_hit_testable(self.element_data.internal_id, bounds);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        // Take focus if clicked.
        if let EventKind::PointerDown(pointer_button) = event {
            self.focus();
            if pointer_button.button == Some(PointerButton::Left) {
                pointer_button.stop_propagation();
            }
        }

        if self.handle_keyboard_input(event) {
            return;
        }

        let list_layout = &self.floating_window.layout;
        let list_box = list_layout.world_box().border_rectangle();
        let list_scroll_box = list_layout.world_scroll_track();

        if self.update_most_recently_hovered_child(event, list_box, list_scroll_box) {
            self.request_window_redraw();
        }

        let pointer_id = event.pointer_id();
        let is_scrolling = self.floating_window.layout.scroll_state.scroll_click.is_some();
        if let EventKind::PointerUp(pb) = event
            && !is_scrolling
        {
            let pointer_position = pb.state.logical_point();
            let is_pointer_in_select_box = self
                .element_data
                .layout
                .world_box()
                .border_rectangle()
                .contains(&pointer_position);
            let is_pointer_in_window = self
                .floating_window
                .layout
                .world_box()
                .border_rectangle()
                .contains(&pointer_position);
            let is_pointer_in_scrollbar = self
                .floating_window
                .layout
                .world_scroll_track()
                .contains(&pointer_position);

            let pointer_id = pointer_id.unwrap();
            if is_pointer_in_select_box {
                self.toggle_menu(&pointer_id);
            } else if !self.is_floating_window_hidden {
                if is_pointer_in_window {
                    self.handle_child_click(&pointer_position, is_pointer_in_scrollbar, &pointer_id);
                } else {
                    self.close_menu();
                }
            }
        }

        // Handle updating the scroll state.
        // TODO: The dropdown scroll logic needs refactoring.
        let floating_window = &mut self.floating_window;
        let result = handle_scroll_logic_advance(&floating_window.style, &mut floating_window.layout, event);
        if result.scroll_changed {
            self.request_window_redraw();
        }
        if result.set_pointer_capture {
            self.set_pointer_capture(result.pointer_id.unwrap())
        } else if result.release_pointer_capture {
            self.release_pointer_capture(result.pointer_id.unwrap());
        }
    }

    fn push(&mut self, child: DynElement) {
        let child = child.inner;
        let me: Weak<RefCell<dyn ElementInternals>> = self.element_data.me.clone();
        let me_window = self.element_data.window.clone();
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(DynElement::new(child.clone()));
        child.borrow_mut().element_data_mut().window = me_window;
        child.borrow_mut().propagate_window_down();
        child
            .borrow_mut()
            .set_scale_factor(self.element_data.applied_scale_factor);

        // Add the children to the floating window layout.
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let parent_id = self.floating_window.layout.gummy_node_id.unwrap();
            let child_id = child.borrow().element_data().layout.gummy_node_id();
            gummy_tree.add_child(parent_id, child_id);
        });
        self.request_window_redraw();
    }

    fn draw_children(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        let floating_transform = renderer.get_transform();
        let scroll_y = self.floating_window.layout.scroll_state.scroll_y() as f64 * scale_factor;
        renderer.set_transform(floating_transform * Affine::translate((0.0, -scroll_y)));

        for (index, child) in self.element_data.children.iter().enumerate() {
            let floating_window_box = self.floating_window.layout.computed_box;
            let mut child_rect = child
                .inner
                .borrow_mut()
                .element_data()
                .layout
                .local_box_in_parent()
                .border_rectangle();

            child_rect.x = 0.0;
            child_rect.width = floating_window_box.size.width;

            let is_hovered = self.currently_hovered_element == Some(index);
            if is_hovered {
                renderer.draw_rect(
                    child_rect.scale(scale_factor),
                    self.hovered_bg_brush.as_ref().unwrap().clone(),
                );
            }

            child
                .inner
                .borrow_mut()
                .draw_transformed(renderer, resource_manager.clone(), scale_factor, text_context);
        }
        renderer.set_transform(floating_transform);
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout.world_box().border_rectangle();
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
}

impl Shape {
    pub fn new(is_scrollable: bool) -> Self {
        let layout = Layout::new(is_scrollable);
        let style = Box::new(Style::new());

        Self { layout, style }
    }

    pub fn create_gummy_node(&mut self) {
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let style = self.style.to_gummy_style();
            let node_id = gummy_tree.new_leaf(style);
            self.layout.gummy_node_id = Some(node_id);
        });
    }

    fn apply_borders(&mut self, scale_factor: f64) {
        let style = &self.style;
        self.layout.apply_borders(
            style.has_border(),
            style.get_border_radius(),
            scale_factor,
            style.get_border_color(),
            style.get_outline_width_px(),
            style.get_box_shadows().to_vec(),
        );
    }

    pub fn apply_simple_layout(&mut self, gummy_tree: &mut GummyTree, z_index: &mut u32, scale_factor: f64) {
        let node = self.layout.gummy_node_id();
        let gummy_layout = gummy_tree.get_layout(node);
        self.layout.has_new_layout = gummy_tree.has_new_layout(node);

        let has_new_layout = self.layout.has_new_layout;

        if has_new_layout {
            self.layout.resolve_box(gummy_layout, z_index);
            self.apply_borders(scale_factor);

            // For scroll changes from gummy;
            apply_scroll_layout(&self.style, &mut self.layout, gummy_layout);
            self.layout.scroll_state.mark_old();
        }

        // For manual scroll updates.
        if !has_new_layout && self.layout.scroll_state.is_new() {
            apply_scroll_layout(&self.style, &mut self.layout, gummy_layout);
            self.layout.scroll_state.mark_old();
        }

        if has_new_layout {
            gummy_tree.mark_seen(node);
        }
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
                hovered_bg_brush: Some(Brush::Color(Color::from_rgba8(213, 213, 215, 255))),
            })
        });

        let border_color = rgba(0, 0, 0, 64);
        let border_width = px(1.0);
        let border_radius = [(5.0, 5.0); 4];

        inner.borrow_mut().element_data.create_layout_node(None);
        inner
            .borrow_mut()
            .element_data
            .set_accessibility_role(issho::Role::ComboBox);
        inner.borrow_mut().element_data.style.set_display(Display::Flex);

        inner
            .borrow_mut()
            .element_data
            .style
            .set_align_items(AlignItems::Center);
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
            .set_background_brush(Brush::Color(Color::WHITE));
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
        inner.borrow_mut().floating_window.create_gummy_node();

        inner.borrow_mut().arrow.style.set_width(px(12.0));
        inner.borrow_mut().arrow.style.set_height(px(6.0));
        inner
            .borrow_mut()
            .arrow
            .style
            .set_margin(TrblRectangle::new(px(0.0), px(8.0), px(0.0), auto()));
        inner.borrow_mut().arrow.create_gummy_node();

        // Set the floating window's parent and the arrow to the Dropdown element.
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let parent_id = inner.borrow_mut().element_data.layout.gummy_node_id();
            let floating_window_child_id = inner.borrow_mut().floating_window.layout.gummy_node_id();
            let arrow_child_id = inner.borrow_mut().arrow.layout.gummy_node_id();
            gummy_tree.add_child(parent_id, floating_window_child_id);
            gummy_tree.add_child(parent_id, arrow_child_id);

            let owner: Rc<RefCell<dyn ElementInternals>> = inner.clone();
            let owner = Rc::downgrade(&owner);
            let owner_id = inner.borrow().element_data.internal_id;
            gummy_tree.register_owner(floating_window_child_id, owner_id, owner.clone());
            gummy_tree.register_owner(arrow_child_id, owner_id, owner);
        });

        Self { inner }
    }

    pub fn selected_item(self, index: usize) -> Self {
        let binding = self.inner.clone();
        let mut inner = binding.borrow_mut();
        inner.set_selected_element(index);
        self
    }

    pub fn get_selected_item(self) -> Option<usize> {
        self.inner.borrow().selected_element_index
    }
}

impl DropdownInner {
    fn handle_keyboard_input(&mut self, event: &mut EventKind) -> bool {
        if !self.is_focused() {
            return false;
        }

        let EventKind::KeyDown(keyboard_event) = event else {
            return false;
        };

        let item_count = self.element_data.children.len();
        let handled = match keyboard_event.code {
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space if !keyboard_event.repeat => {
                if self.is_floating_window_hidden {
                    self.open_menu();
                } else {
                    if let Some(index) = self.currently_hovered_element.filter(|index| *index < item_count) {
                        self.set_selected_element(index);
                        self.queue_dropdown_item_selected(index);
                    }
                    self.close_menu();
                }
                true
            }
            KeyCode::Escape if !self.is_floating_window_hidden => {
                self.close_menu();
                true
            }
            KeyCode::ArrowDown if item_count > 0 => {
                self.move_keyboard_selection(1, item_count);
                true
            }
            KeyCode::ArrowUp if item_count > 0 => {
                self.move_keyboard_selection(-1, item_count);
                true
            }
            KeyCode::Home if item_count > 0 => {
                self.set_keyboard_selection(0);
                true
            }
            KeyCode::End if item_count > 0 => {
                self.set_keyboard_selection(item_count - 1);
                true
            }
            _ => false,
        };

        if handled {
            keyboard_event.stop_propagation();
            keyboard_event.prevent_default();
        }
        handled
    }

    fn move_keyboard_selection(&mut self, direction: isize, item_count: usize) {
        let current = if self.is_floating_window_hidden {
            self.selected_element_index
        } else {
            self.currently_hovered_element.or(self.selected_element_index)
        };
        let next = match current {
            Some(index) => index.saturating_add_signed(direction).min(item_count - 1),
            None if direction < 0 => item_count - 1,
            None => 0,
        };
        self.set_keyboard_selection(next);
    }

    fn set_keyboard_selection(&mut self, index: usize) {
        if self.is_floating_window_hidden {
            if self.selected_element_index != Some(index) {
                self.set_selected_element(index);
                self.queue_dropdown_item_selected(index);
            }
        } else {
            let highlight_changed = self.currently_hovered_element != Some(index);
            self.currently_hovered_element = Some(index);
            let scroll_changed = self.scroll_item_into_view(index);
            if highlight_changed || scroll_changed {
                self.request_window_redraw();
            }
        }
    }

    fn scroll_item_into_view(&mut self, index: usize) -> bool {
        let Some(item) = self.element_data.children.get(index) else {
            return false;
        };
        let item_box = item
            .borrow()
            .element_data()
            .layout
            .local_box_in_parent()
            .border_rectangle();

        let layout = &mut self.floating_window.layout;
        let viewport = layout.local_box().padding_rectangle();
        let current_scroll_y = layout.scroll_state.scroll_y();
        let visible_top = viewport.top() + current_scroll_y;
        let visible_bottom = viewport.bottom() + current_scroll_y;
        let target_scroll_y = if item_box.top() < visible_top {
            item_box.top() - viewport.top()
        } else if item_box.bottom() > visible_bottom {
            item_box.bottom() - viewport.bottom()
        } else {
            return false;
        };

        set_scroll_y(layout, target_scroll_y)
    }

    fn draw_selected_element(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        if let Some(selected_element) = &self.selected_element {
            // This clone is a presentation-only preview. Its subtree must not
            // intercept pointer events intended for the dropdown itself.
            let target_count = renderer.render_list().targets.len();
            let mut binding = selected_element.borrow_mut();
            binding.draw_transformed(renderer, resource_manager.clone(), scale_factor, text_context);
            renderer.render_list_mut().targets.truncate(target_count);
        }
    }

    fn set_selected_element(&mut self, child_index: usize) {
        // Remove the old selected element from the layout tree.
        if let Some(old_selected_element) = &self.selected_element {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.unparent_node(old_selected_element.borrow().element_data().layout.gummy_node_id());
            });
        }

        let child = self
            .element_data
            .children
            .get(child_index)
            .expect("There is no child at this index.");
        self.selected_element = Some(child.clone().borrow().deep_clone().inner);
        self.selected_element
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_scale_factor(self.element_data.applied_scale_factor);
        self.selected_element_index = Some(child_index);
        let selected_element_id = self
            .selected_element
            .as_ref()
            .unwrap()
            .borrow()
            .element_data()
            .layout
            .gummy_node_id();

        // Add the selected element to the parent's layout tree at index 1.
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let parent_id = self.element_data.layout.gummy_node_id.unwrap();
            gummy_tree.add_child_at_index(parent_id, selected_element_id, 1);
        });
        self.request_window_redraw();
    }

    fn update_most_recently_hovered_child(
        &mut self,
        message: &EventKind,
        list_box: Rectangle,
        list_scroll_box: Rectangle,
    ) -> bool {
        if self.is_floating_window_hidden {
            return false;
        }
        let previous = self.currently_hovered_element;
        if let EventKind::PointerMoved(pb) = message {
            let pointer_position = pb.current.logical_point();
            let is_pointer_in_list = list_box.contains(&pointer_position);
            let is_pointer_in_scrollbar = list_scroll_box.contains(&pointer_position);

            if is_pointer_in_list && !is_pointer_in_scrollbar {
                let hovered_child = self
                    .element_data
                    .children
                    .iter()
                    .enumerate()
                    .find_map(|(index, child)| {
                        let contains = child
                            .borrow()
                            .element_data()
                            .layout
                            .world_box()
                            .border_rectangle()
                            .contains(&pointer_position);

                        if contains {
                            return Some(index);
                        }

                        None
                    });

                self.currently_hovered_element = hovered_child;
            } else {
                self.currently_hovered_element = None;
            }
        }
        previous != self.currently_hovered_element
    }

    fn toggle_menu(&mut self, pointer_id: &PointerId) {
        if self.is_floating_window_hidden {
            self.open_menu();
        } else {
            self.close_menu();
            self.release_pointer_capture(*pointer_id);
        }
    }

    fn open_menu(&mut self) {
        self.is_floating_window_hidden = false;
        self.currently_hovered_element = self
            .selected_element_index
            .or_else(|| (!self.element_data.children.is_empty()).then_some(0));
        self.queue_dropdown_toggled(true);
        self.request_window_redraw();
    }

    fn close_menu(&mut self) {
        self.is_floating_window_hidden = true;
        self.queue_dropdown_toggled(false);
        self.request_window_redraw();
    }

    fn handle_child_click(&mut self, pointer_position: &Point, is_pointer_in_scrollbar: bool, pointer_id: &PointerId) {
        if !is_pointer_in_scrollbar {
            let mut should_hide_window = false;
            for (child_index, child) in self.element_data.children.iter().cloned().enumerate() {
                let contains = child
                    .borrow()
                    .element_data()
                    .layout
                    .world_box()
                    .border_rectangle()
                    .contains(pointer_position);

                if contains {
                    should_hide_window = true;
                    self.set_selected_element(child_index);
                    self.release_pointer_capture(*pointer_id);

                    self.queue_dropdown_item_selected(child_index);

                    break;
                }
            }

            if should_hide_window {
                self.is_floating_window_hidden = true;
                self.queue_dropdown_toggled(false);
                self.request_window_redraw();
            }
        }
    }

    fn queue_dropdown_toggled(&self, is_open: bool) {
        let target = self.element_data.me.upgrade().unwrap();
        queue_event(EventKind::DropdownToggled(DropdownToggledEvent::new(
            DynElement::new(target),
            is_open,
        )));
    }

    fn queue_dropdown_item_selected(&self, index: usize) {
        let target = self.element_data.me.upgrade().unwrap();
        queue_event(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
            DynElement::new(target),
            index,
        )));
    }
}
