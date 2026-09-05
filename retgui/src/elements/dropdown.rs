//! An element to select a single item from a collapsable vertical list of options.

use retgui_primitives::geometry::{BezPath, Rectangle, TrblRectangle};
use std::any::Any;
use std::sync::Arc;

use retgui_renderer::Brush;

use retgui_primitives::geometry::{Affine, Point, Vec2};

use peniko::Color;

use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData as ElementDataStruct;
use crate::elements::scrollable::{apply_scroll_layout, draw_scrollbar, handle_scroll_logic_advance, set_scroll_y};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementInternals, Elements, HasElementData};
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
/// use retgui::elements::{Dropdown, Element, Elements, Text, Window};
/// use retgui::{RetGuiOptions, px, retgui_main};
///
/// fn main() {
///     let mut elements = Elements::new();
///     let item_1 = Text::new(&mut elements, "Item 1")
///         .edit(&mut elements)
///         .font_size(20.0)
///         .selectable(false)
///         .finish();
///     let item_2 = Text::new(&mut elements, "Item 2")
///         .edit(&mut elements)
///         .font_size(20.0)
///         .selectable(false)
///         .finish();
///     let item_3 = Text::new(&mut elements, "Item 3")
///         .edit(&mut elements)
///         .font_size(20.0)
///         .selectable(false)
///         .finish();
///
///     let dropdown = Dropdown::new(&mut elements)
///         .edit(&mut elements)
///         .width(px(100))
///         .push(item_1)
///         .push(item_2)
///         .push(item_3)
///         .selected_item(0)
///         .finish();
///     Window::new(&mut elements, "Dropdown")
///         .edit(&mut elements)
///         .push(dropdown)
///         .finish();
///     retgui_main(elements, RetGuiOptions::basic("Dropdown"));
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Dropdown {
    pub(crate) inner: DynElement,
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
pub(crate) struct DropdownElement {
    element_data: ElementDataStruct,
    floating_window: Shape,
    arrow: Shape,
    is_floating_window_hidden: bool,
    selected_element: Option<DynElement>,
    selected_element_index: Option<usize>,
    currently_hovered_element: Option<usize>,
    hovered_bg_brush: Option<Brush>,
}

impl Element for Dropdown {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl HasElementData for DropdownElement {
    fn element_data(&self) -> &ElementDataStruct {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementDataStruct {
        &mut self.element_data
    }
}

impl ElementInternals for DropdownElement {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        let element = clone_element::<Self, _>(self, elements, |element, gummy_tree| {
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me;
            let parent = element.element_data.layout.gummy_node_id();
            let floating_window_node = gummy_tree.clone_node(element.floating_window.layout.gummy_node_id());
            let arrow_node = gummy_tree.clone_node(element.arrow.layout.gummy_node_id());

            element.floating_window.layout.gummy_node_id = Some(floating_window_node);
            element.arrow.layout.gummy_node_id = Some(arrow_node);
            element.selected_element = None;

            gummy_tree.add_child(parent, floating_window_node);
            gummy_tree.add_child(parent, arrow_node);
            gummy_tree.register_owner(floating_window_node, owner_id, owner);
            gummy_tree.register_owner(arrow_node, owner_id, owner);
            Some(floating_window_node)
        });
        let selected_element_index = elements.get_as::<Self>(element).selected_element_index;
        if let Some(index) = selected_element_index {
            elements.dispatch_mut(element, |element, elements| {
                (element as &mut dyn Any)
                    .downcast_mut::<Self>()
                    .unwrap()
                    .set_selected_element(elements, index)
            });
        }
        DynElement::new(element)
    }

    fn set_scale_factor(&mut self, elements: &mut Elements, scale_factor: f64) {
        self.element_data.applied_scale_factor = scale_factor;
        self.apply_borders(scale_factor);
        self.floating_window.apply_borders(scale_factor);
        self.arrow.apply_borders(scale_factor);
        for child in self.element_data.children.clone() {
            elements.dispatch_mut(child, |child, elements| child.set_scale_factor(elements, scale_factor));
        }
        if let Some(selected_element) = &self.selected_element {
            elements.dispatch_mut(*selected_element, |selected, elements| {
                selected.set_scale_factor(elements, scale_factor)
            });
        }
        self.mark_dirty(&mut elements.gummy_tree);
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
        self.element_data
            .layout
            .has_new_layout
            .set(gummy_tree.has_new_layout(node));
        let has_new_layout = self.element_data.layout.has_new_layout.get();

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
        &self,
        elements: &Elements,
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

        self.draw_selected_element(elements, renderer, resource_manager.clone(), text_context, scale_factor);

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

            self.draw_children(elements, renderer, resource_manager.clone(), scale_factor, text_context);

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

    fn add_hit_testable(&self, renderer: &mut dyn Renderer, hit_testable: bool, scale_factor: f64) {
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

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        // Take focus if clicked.
        if let EventKind::PointerDown(pointer_button) = event {
            self.focus(elements);
            if pointer_button.button == Some(PointerButton::Left) {
                pointer_button.stop_propagation();
            }
        }

        if self.handle_keyboard_input(elements, event) {
            return;
        }

        let list_layout = &self.floating_window.layout;
        let list_box = list_layout.world_box().border_rectangle();
        let list_scroll_box = list_layout.world_scroll_track();

        if self.update_most_recently_hovered_child(elements, event, list_box, list_scroll_box) {
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
                self.toggle_menu(elements, &pointer_id);
            } else if !self.is_floating_window_hidden {
                if is_pointer_in_window {
                    self.handle_child_click(elements, &pointer_position, is_pointer_in_scrollbar, &pointer_id);
                } else {
                    self.close_menu(elements);
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
            self.set_pointer_capture(elements, result.pointer_id.unwrap())
        } else if result.release_pointer_capture {
            self.release_pointer_capture(elements, result.pointer_id.unwrap());
        }
    }

    fn child_layout_parent(&self) -> Option<gummy::NodeId> {
        self.floating_window.layout.gummy_node_id
    }

    fn draw_children(
        &self,
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        let floating_transform = renderer.get_transform();
        let scroll_y = self.floating_window.layout.scroll_state.scroll_y() as f64 * scale_factor;
        renderer.set_transform(floating_transform * Affine::translate((0.0, -scroll_y)));

        for (index, child) in self.element_data.children.iter().copied().enumerate() {
            let floating_window_box = self.floating_window.layout.computed_box;
            let mut child_rect = elements
                .get(child)
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

            elements.get_for_draw(child).draw_transformed(
                elements,
                renderer,
                resource_manager.clone(),
                scale_factor,
                text_context,
            );
        }
        renderer.set_transform(floating_transform);
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout.world_box().border_rectangle();
        if !self.is_floating_window_hidden {
            return true;
        }

        if let Some(clip) = element_data.layout.clip_bounds.get() {
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

    pub fn create_gummy_node(&mut self, gummy_tree: &mut GummyTree) {
        let style = self.style.to_gummy_style();
        let node_id = gummy_tree.new_leaf(style);
        self.layout.gummy_node_id = Some(node_id);
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
        self.layout.has_new_layout.set(gummy_tree.has_new_layout(node));

        let has_new_layout = self.layout.has_new_layout.get();

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
    pub fn new(elements: &mut Elements) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(DropdownElement {
                element_data: ElementDataStruct::new(me, true, access_tree),
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

        let element = elements.get_as_mut::<DropdownElement>(inner);
        element.element_data.set_accessibility_role(issho::Role::ComboBox);
        element.element_data.style.set_display(Display::Flex);
        element.element_data.style.set_align_items(AlignItems::Center);
        element
            .element_data
            .style
            .set_padding(TrblRectangle::new(px(2.5), px(0.0), px(2.5), px(6.0)));
        element
            .element_data
            .style
            .set_border_width(TrblRectangle::new_all(border_width));
        element.element_data.style.set_border_radius(border_radius);
        element
            .element_data
            .style
            .set_border_color(TrblRectangle::new_all(border_color));

        element
            .floating_window
            .style
            .set_background_brush(Brush::Color(Color::WHITE));
        element.floating_window.style.set_position(Position::Absolute);
        element.floating_window.style.set_display(Display::Flex);
        element.floating_window.style.set_flex_direction(FlexDirection::Column);
        element.floating_window.style.set_box_shadows(vec![BoxShadow::new(
            false,
            0.0,
            4.0,
            8.0,
            1.0,
            rgba(0, 0, 0, 255),
        )]);
        element
            .floating_window
            .style
            .set_padding(TrblRectangle::new(px(2.5), px(0.0), px(2.5), px(6.0)));
        element.floating_window.style.set_width(Unit::Percentage(100.0));
        element
            .floating_window
            .style
            .set_overflow([Overflow::Visible, Overflow::Scroll]);
        element.floating_window.style.set_height(px(120.0));
        element.floating_window.style.set_max_height(px(100.0));
        element
            .floating_window
            .style
            .set_border_width(TrblRectangle::new_all(border_width));
        element.floating_window.style.set_border_radius(border_radius);
        element
            .floating_window
            .style
            .set_border_color(TrblRectangle::new_all(border_color));

        element.arrow.style.set_width(px(12.0));
        element.arrow.style.set_height(px(6.0));
        element
            .arrow
            .style
            .set_margin(TrblRectangle::new(px(0.0), px(8.0), px(0.0), auto()));
        let _ = element;
        {
            let (gummy_tree, elements) = elements.disjoint_borrow_layout_and_elements();
            let element = elements.get_as_mut::<DropdownElement>(inner);
            element.element_data.create_layout_node(gummy_tree, None);
            element.floating_window.create_gummy_node(gummy_tree);
            element.arrow.create_gummy_node(gummy_tree);
            let parent_id = element.element_data.layout.gummy_node_id();
            let floating_window_child_id = element.floating_window.layout.gummy_node_id();
            let arrow_child_id = element.arrow.layout.gummy_node_id();
            gummy_tree.add_child(parent_id, floating_window_child_id);
            gummy_tree.add_child(parent_id, arrow_child_id);

            let owner_id = element.element_data.internal_id;
            gummy_tree.register_owner(floating_window_child_id, owner_id, inner);
            gummy_tree.register_owner(arrow_child_id, owner_id, inner);
        }

        Self { inner }
    }

    pub fn set_selected_item(&self, elements: &mut Elements, index: usize) {
        elements.try_dispatch_mut(self.inner, |inner, elements| {
            let inner = (inner as &mut dyn Any).downcast_mut::<DropdownElement>().unwrap();
            inner.set_selected_element(elements, index);
        });
    }

    pub fn selected_item(&self, elements: &Elements) -> Option<usize> {
        elements
            .try_get_as::<DropdownElement>(self.inner)
            .and_then(|dropdown| dropdown.selected_element_index)
    }
}

impl DropdownElement {
    fn handle_keyboard_input(&mut self, elements: &mut Elements, event: &mut EventKind) -> bool {
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
                    self.open_menu(elements);
                } else {
                    if let Some(index) = self.currently_hovered_element.filter(|index| *index < item_count) {
                        self.set_selected_element(elements, index);
                        self.queue_dropdown_item_selected(elements, index);
                    }
                    self.close_menu(elements);
                }
                true
            }
            KeyCode::Escape if !self.is_floating_window_hidden => {
                self.close_menu(elements);
                true
            }
            KeyCode::ArrowDown if item_count > 0 => {
                self.move_keyboard_selection(elements, 1, item_count);
                true
            }
            KeyCode::ArrowUp if item_count > 0 => {
                self.move_keyboard_selection(elements, -1, item_count);
                true
            }
            KeyCode::Home if item_count > 0 => {
                self.set_keyboard_selection(elements, 0);
                true
            }
            KeyCode::End if item_count > 0 => {
                self.set_keyboard_selection(elements, item_count - 1);
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

    fn move_keyboard_selection(&mut self, elements: &mut Elements, direction: isize, item_count: usize) {
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
        self.set_keyboard_selection(elements, next);
    }

    fn set_keyboard_selection(&mut self, elements: &mut Elements, index: usize) {
        if self.is_floating_window_hidden {
            if self.selected_element_index != Some(index) {
                self.set_selected_element(elements, index);
                self.queue_dropdown_item_selected(elements, index);
            }
        } else {
            let highlight_changed = self.currently_hovered_element != Some(index);
            self.currently_hovered_element = Some(index);
            let scroll_changed = self.scroll_item_into_view(elements, index);
            if highlight_changed || scroll_changed {
                self.request_window_redraw();
            }
        }
    }

    fn scroll_item_into_view(&mut self, elements: &Elements, index: usize) -> bool {
        let Some(item) = self.element_data.children.get(index) else {
            return false;
        };
        let item_box = elements
            .get(*item)
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
        &self,
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        if let Some(selected_element) = &self.selected_element {
            // This clone is a presentation-only preview. Its subtree must not
            // intercept pointer events intended for the dropdown itself.
            let target_count = renderer.render_list().targets.len();
            elements.get_for_draw(*selected_element).draw_transformed(
                elements,
                renderer,
                resource_manager.clone(),
                scale_factor,
                text_context,
            );
            renderer.render_list_mut().targets.truncate(target_count);
        }
    }

    fn set_selected_element(&mut self, elements: &mut Elements, child_index: usize) {
        // Remove the old selected element from the layout tree.
        if let Some(old_selected_element) = &self.selected_element {
            let old_node = elements
                .get(*old_selected_element)
                .element_data()
                .layout
                .gummy_node_id();
            elements.gummy_tree.unparent_node(old_node);
        }

        let child = self
            .element_data
            .children
            .get(child_index)
            .expect("There is no child at this index.");
        let child = *child;
        self.selected_element = Some(elements.dispatch_mut(child, |child, elements| child.deep_clone(elements)));
        let selected = self.selected_element.unwrap();
        let scale = self.element_data.applied_scale_factor;
        elements.dispatch_mut(selected, |selected, elements| {
            selected.set_scale_factor(elements, scale)
        });
        self.selected_element_index = Some(child_index);
        let selected_element_id = elements
            .get(self.selected_element.unwrap())
            .element_data()
            .layout
            .gummy_node_id();

        // Add the selected element to the parent's layout tree at index 1.
        let parent_id = self.element_data.layout.gummy_node_id.unwrap();
        elements
            .gummy_tree
            .add_child_at_index(parent_id, selected_element_id, 1);
        self.request_window_redraw();
    }

    fn update_most_recently_hovered_child(
        &mut self,
        elements: &Elements,
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
                        let contains = elements
                            .get(*child)
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

    fn toggle_menu(&mut self, elements: &mut Elements, pointer_id: &PointerId) {
        if self.is_floating_window_hidden {
            self.open_menu(elements);
        } else {
            self.close_menu(elements);
            self.release_pointer_capture(elements, *pointer_id);
        }
    }

    fn open_menu(&mut self, elements: &mut Elements) {
        self.is_floating_window_hidden = false;
        self.currently_hovered_element = self
            .selected_element_index
            .or_else(|| (!self.element_data.children.is_empty()).then_some(0));
        self.queue_dropdown_toggled(elements, true);
        self.request_window_redraw();
    }

    fn close_menu(&mut self, elements: &mut Elements) {
        self.is_floating_window_hidden = true;
        self.queue_dropdown_toggled(elements, false);
        self.request_window_redraw();
    }

    fn handle_child_click(
        &mut self,
        elements: &mut Elements,
        pointer_position: &Point,
        is_pointer_in_scrollbar: bool,
        pointer_id: &PointerId,
    ) {
        if !is_pointer_in_scrollbar {
            let mut should_hide_window = false;
            for (child_index, child) in self.element_data.children.iter().cloned().enumerate() {
                let contains = elements
                    .get(child)
                    .element_data()
                    .layout
                    .world_box()
                    .border_rectangle()
                    .contains(pointer_position);

                if contains {
                    should_hide_window = true;
                    self.set_selected_element(elements, child_index);
                    self.release_pointer_capture(elements, *pointer_id);

                    self.queue_dropdown_item_selected(elements, child_index);

                    break;
                }
            }

            if should_hide_window {
                self.is_floating_window_hidden = true;
                self.queue_dropdown_toggled(elements, false);
                self.request_window_redraw();
            }
        }
    }

    fn queue_dropdown_toggled(&self, elements: &mut Elements, is_open: bool) {
        let target = self.element_data.me;
        elements.queue_event(EventKind::DropdownToggled(DropdownToggledEvent::new(target, is_open)));
    }

    fn queue_dropdown_item_selected(&self, elements: &mut Elements, index: usize) {
        let target = self.element_data.me;
        elements.queue_event(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
            target, index,
        )));
    }
}
