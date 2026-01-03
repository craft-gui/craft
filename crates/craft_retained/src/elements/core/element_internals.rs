use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use accesskit::{Action, Role};
use craft_primitives::geometry::borders::CssRoundedRect;
use craft_primitives::geometry::{ElementBox, Rectangle};
use craft_renderer::RenderList;
use kurbo::{Affine, Point, Vec2};
use rustc_hash::FxHashMap;
use taffy::Overflow;

use crate::animations::animation::{ActiveAnimation, AnimationFlags, AnimationStatus};
use crate::app::{TAFFY_TREE, request_layout};
use crate::elements::Element;
use crate::elements::core::element_data::ElementData;
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::layout::layout_item::{CssComputedBorder, LayoutItem, draw_borders_generic};
use crate::style::{Display, Style};
use crate::text::text_context::TextContext;

/// Internal element methods that should typically be ignored by users. Public for custom elements.
pub trait ElementInternals: ElementData {
    /// A helper to apply the layout for all children.
    fn apply_layout_children(
        &mut self,
        taffy_tree: &mut TaffyTree,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        scale_factor: f64,
        _dirty: bool,
    ) {
        for child in &self.element_data().children {
            child.borrow_mut().apply_layout(
                taffy_tree,
                self.element_data().layout_item.computed_box.position,
                z_index,
                transform,
                pointer,
                text_context,
                self.element_data().layout_item.clip_bounds,
                scale_factor,
            );
        }
    }

    /// A helper to check if the element is visible.
    fn is_visible(&self) -> bool {
        let style = &self.element_data().style;
        style.visible() && style.display() != Display::None
    }

    /// A helper to draw all children.
    fn draw_children(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        scale_factor: f64,
    ) {
        for child in self.children() {
            child.borrow_mut().draw(renderer, text_context, pointer, scale_factor);
        }
    }

    /// A helper to re-apply the style to the layout node when dirty.
    fn apply_style_to_layout_node_if_dirty(&mut self, taffy_tree: &mut TaffyTree) {
        let element_data = self.element_data_mut();
        if element_data.style.is_dirty {
            let node_id = element_data.layout_item.taffy_node_id.unwrap();
            let style: taffy::Style = element_data.style.to_taffy_style();
            taffy_tree.set_style(node_id, style);
            element_data.style.is_dirty = false;
        }
    }

    /// Applies the layout results from the [`TaffyTree`].
    /// This method retrieves the computed layout for `root_node` and updates the
    /// element’s internal state accordingly. It resolves the element's position,
    /// transform, clipping, borders, and stacking order, producing the final
    /// layout state used for rendering.
    ///
    /// # Parameters
    /// - `taffy_tree`: The layout tree containing the computed results.
    /// - `root_node`: The node whose layout information should be applied.
    /// - `position`: The absolute position of the element within its parent context.
    /// - `z_index`: A mutable counter used to assign stacking order as elements
    ///   are processed.
    /// - `transform`: The accumulated transform to apply to this element.
    /// - `pointer`: The current pointer position, if available, for hit-testing.
    /// - `text_context`: Context used for text layout and measurement.
    /// - `clip_bounds`: Optional clipping rectangle inherited from ancestors.
    ///
    /// # Effects
    /// This function mutates internal element state to reflect the final resolved
    /// layout and may trigger updates such as clipping regions, border geometry,
    /// and z-index assignment.
    #[allow(clippy::too_many_arguments)]
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
    );

    /// Draws the element and its visual contents.
    ///
    /// Implementations should use the provided [`RenderList`] to issue
    /// drawing commands.
    ///
    /// - `renderer`: the active render list to draw into.
    /// - `text_context`: text shaping and layout context.
    /// - `pointer`: optional pointer position for hover effects.
    /// - `window`: optional window handle.
    /// - `scale_factor`: scale factor.
    fn draw(
        &mut self,
        _renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        _scale_factor: f64,
    ) {
    }

    /// Computes a [`TreeUpdate`] reflecting any accessibility changes.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let current_node_id = accesskit::NodeId(self.element_data().internal_id);

        let mut current_node = accesskit::Node::new(Role::GenericContainer);
        if !self.element_data().on_pointer_button_up.is_empty() {
            current_node.set_role(Role::Button);
            current_node.add_action(Action::Click);
        }

        let padding_box = self
            .element_data()
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

        for child in self.element_data_mut().children.iter_mut() {
            child
                .borrow_mut()
                .compute_accessibility_tree(tree, Some(current_index), scale_factor);
        }
    }

    /// Handles default events.
    fn on_event(
        &mut self,
        _message: &CraftMessage,
        _text_context: &mut TextContext,
        _event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
    }

    /// Computes this element's box model.
    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: Affine,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
        let position = self.element_data().style.position();
        self.element_data_mut().layout_item.resolve_box(
            relative_position,
            scroll_transform,
            result,
            layout_order,
            position,
        );
    }

    /// Computes this element's clip box.
    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.element_data_mut().layout_item.resolve_clip(clip_bounds);
    }

    fn apply_borders(&mut self, scale_factor: f64) {
        let current_style = self.element_data().current_style();
        let has_border = current_style.has_border();
        let border_radius = current_style.border_radius();
        let border_color = &current_style.border_color();

        self.element_data_mut()
            .layout_item
            .apply_borders(has_border, border_radius, scale_factor, border_color);
    }

    /// Called after layout, and is responsible for updating the animation state of an element.
    fn on_animation_frame(&mut self, animation_flags: &mut AnimationFlags, delta_time: Duration) {
        let base_state = self.element_data_mut();

        // If we don't have an animation in the current style then try to fall back to the normal style.
        let current_style = if let Some(current_style) = base_state.current_style_mut_no_fallback() {
            current_style
        } else {
            &mut base_state.style
        };

        // This is pretty hacky, but we can avoid allocating a hashmap for every element.
        let active_animations = {
            if base_state.animations.is_none() {
                base_state.animations = Some(FxHashMap::default());
            }

            base_state.animations.as_mut().unwrap()
        };

        let current_style_animations = &mut current_style.animations;
        for ani in &mut *current_style_animations {
            if !active_animations.contains_key(&ani.name) {
                active_animations.insert(
                    ani.name.clone(),
                    ActiveAnimation {
                        current: Duration::ZERO,
                        status: AnimationStatus::Playing,
                        loop_amount: ani.loop_amount,
                    },
                );
            }
        }

        active_animations.retain(|key, _| current_style_animations.iter().any(|ani| &ani.name == key));

        active_animations.retain(|anim_name, active_animation| {
            if active_animation.status == AnimationStatus::Playing {
                animation_flags.set_has_active_animation(true);
            }

            if let Some(animation) = current_style.animation(anim_name) {
                active_animation.tick(animation_flags, animation, delta_time);
                let new_style = active_animation.compute_style(current_style, animation, animation_flags);
                *current_style = Style::merge(current_style, &new_style);
                true
            } else {
                false
            }
        });

        for child in self.children() {
            child.borrow_mut().on_animation_frame(animation_flags, delta_time);
        }
    }

    /// A bit of a hack to reset the layout item of an element recursively.
    fn reset_layout_item(&mut self) {
        self.element_data_mut().layout_item = LayoutItem::default();

        for child in self.element_data_mut().children.iter_mut() {
            child.borrow_mut().reset_layout_item();
        }
    }

    fn add_hit_testable(&mut self, renderer: &mut RenderList, hit_testable: bool, scale_factor: f64) {
        /*let ed = self.element_data().borrow();
        let has_events =
            !ed.on_pointer_button_up.is_empty() ||
            !ed.on_pointer_moved.is_empty() ||
            !ed.on_keyboard_input.is_empty() ||
            !ed.on_pointer_button_down.is_empty() ||
            !ed.on_got_pointer_capture.is_empty() ||
            !ed.on_pointer_enter.is_empty() ||
            !ed.on_pointer_leave.is_empty() ||
            !ed.on_lost_pointer_capture;*/
        if hit_testable {
            let id = self.element_data().internal_id;
            renderer.push_hit_testable(
                id,
                self.element_data()
                    .layout_item
                    .computed_box_transformed
                    .padding_rectangle()
                    .scale(scale_factor),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut RenderList, scale_factor: f64) {
        let current_style = self.element_data().current_style();

        self.element_data()
            .layout_item
            .draw_borders(renderer, current_style, scale_factor);
    }

    fn maybe_start_layer(&self, renderer: &mut RenderList, scale_factor: f64) {
        let element_data = self.element_data();
        let padding_rectangle = element_data
            .layout_item
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut RenderList) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn draw_scrollbar(&mut self, renderer: &mut RenderList, scale_factor: f64) {
        if !self.element_data().is_scrollable() {
            return;
        }

        let border_color = self.element_data().current_style().border_color();
        let scrollbar_color = self.element_data().current_style().scrollbar_color();
        let scrollbar_thumb_radius = self
            .element_data()
            .current_style()
            .scrollbar_thumb_radius()
            .map(|radii| Vec2::new(radii.0 as f64, radii.1 as f64));
        // let scrollbar_thumb_radius = self.element_data().current_style().
        let track_rect = self
            .element_data_mut()
            .layout_item
            .computed_scroll_track
            .scale(scale_factor);
        let thumb_rect = self
            .element_data_mut()
            .layout_item
            .computed_scroll_thumb
            .scale(scale_factor);

        let border_spec = CssRoundedRect::new(thumb_rect.to_kurbo(), [0.0, 0.0, 0.0, 0.0], scrollbar_thumb_radius);
        let mut computed_border_spec = CssComputedBorder::new(border_spec);
        computed_border_spec.scale(scale_factor);

        renderer.draw_rect(track_rect, scrollbar_color.track_color);
        draw_borders_generic(
            renderer,
            &computed_border_spec,
            border_color.to_array(),
            scrollbar_color.thumb_color,
        );
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.current_style().overflow()[1] == Overflow::Scroll
    }

    /// Returns the element's [`ElementBox`] without any transforms applied.
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout_item.computed_box
    }

    /// Gets
    fn get_default_style() -> Style
    where
        Self: Sized,
    {
        Style::default()
    }

    /// Mark layout node dirty.
    fn mark_dirty(&mut self) {
        request_layout();
        let id = self.element_data().layout_item.taffy_node_id;
        if let Some(id) = id {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.mark_dirty(id);
            });
        }
    }

    /// Updates taffy's style to reflect craft's style struct.
    fn update_taffy_style(&mut self) {
        request_layout();

        let id = self.element_data().layout_item.taffy_node_id;
        if let Some(id) = id {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.set_style(id, self.element_data().style.to_taffy_style());
            });
        }
    }

    /// Set's this element's scale factor. This should not be used to scale individual elements.
    fn scale_factor(&mut self, scale_factor: f64) {
        for child in &self.element_data().children {
            child.borrow_mut().scale_factor(scale_factor);
        }
    }
}

pub(crate) fn resolve_clip_for_scrollable(element: &mut dyn Element, clip_bounds: Option<Rectangle>) {
    let element_data = element.element_data_mut();
    if element_data.is_scrollable() {
        let scroll_clip_bounds = element_data.layout_item.computed_box_transformed.padding_rectangle();
        if let Some(clip_bounds) = clip_bounds {
            element_data.layout_item.clip_bounds = scroll_clip_bounds.intersection(&clip_bounds);
        } else {
            element_data.layout_item.clip_bounds = Some(scroll_clip_bounds);
        }
    } else {
        element_data.layout_item.clip_bounds = clip_bounds;
    }
}
