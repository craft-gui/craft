use crate::animations::animation::{ActiveAnimation, AnimationFlags, AnimationStatus};
use crate::elements::element_states::ElementState;
use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::LayoutContext;
use crate::layout::layout_item::{draw_borders_generic, LayoutItem};
use crate::style::Style;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::borders::BorderSpec;
use craft_primitives::geometry::{ElementBox, Rectangle, TrblRectangle};
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use peniko::Color;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use taffy::{NodeId, Overflow, TaffyTree};
use winit::window::Window;

use crate::elements::core::element_data::ElementData;
use crate::elements::Element;
#[cfg(feature = "accesskit")]
use accesskit::{Action, Role};

/// Internal element methods that should typically be ignored by users. Public for custom elements.
pub trait ElementInternals: ElementData {

    /// Construct the [`TaffyTree`].
    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, scale_factor: f64) -> Option<NodeId>;

    /// Updates the element to reflect the layour results from the [`TaffyTree`].
    ///
    /// The majority of the layout computation is done in the `compute_layout` method.
    /// Store the computed values in the `element_data` struct.
    #[allow(clippy::too_many_arguments)]
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
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
    }

    /// Computes a [`TreeUpdate`] reflecting any accessibility changes.
    #[cfg(feature = "accesskit")]
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

        let padding_box =
            self.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);

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
            child.borrow_mut().compute_accessibility_tree(tree, Some(current_index), scale_factor);
        }
    }

    /// Handles default events.
    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<Rc<RefCell<dyn ElementInternals>>>,
        //_current_target: Option<&dyn Element>,
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
    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.element_data_mut().layout_item.resolve_clip(clip_bounds);
    }

    fn finalize_borders(&mut self) {
        let (has_border, border_radius, border_color) = {
            let current_style = self.element_data().current_style();
            (current_style.has_border(), current_style.border_radius(), current_style.border_color())
        };

        self.element_data_mut().layout_item.finalize_borders(has_border, border_radius, border_color);
    }

    /// Called after layout, and is responsible for updating the animation state of an element.
    fn on_animation_frame(&mut self, animation_flags: &mut AnimationFlags, delta_time: Duration) {
        let mut base_state = self.element_data_mut();
        let current_state: ElementState = {
            if base_state.hovered {
                ElementState::Hovered
            } else if base_state.focused {
                ElementState::Focused
            } else {
                ElementState::Normal
            }
        };

        // If we don't have an animation in the current style then try to fall back to the normal style.
        let current_style =
            if let Some(current_style) = base_state.current_style_mut_no_fallback() {
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
                active_animations.insert(ani.name.clone(), ActiveAnimation {
                    current: Duration::ZERO,
                    status: AnimationStatus::Playing,
                    loop_amount: ani.loop_amount,
                });
            }
        }

        active_animations.retain(|key, _| {
            current_style_animations.iter().any(|ani| &ani.name == key)
        });

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

        for child in self.children_mut() {
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

    fn draw_borders(&self, renderer: &mut RenderList, scale_factor: f64) {
        let current_style = self.element_data().current_style();

        self.element_data().layout_item.draw_borders(renderer, current_style, scale_factor);
    }

    fn maybe_start_layer(&self, renderer: &mut RenderList, scale_factor: f64) {
        let element_data = self.element_data();
        let padding_rectangle =
            element_data.layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);

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

        let scrollbar_color = self.element_data().current_style().scrollbar_color();
        let scrollbar_thumb_radius = self.element_data().current_style().scrollbar_thumb_radius();
        // let scrollbar_thumb_radius = self.element_data().current_style().
        let track_rect = self.element_data_mut().layout_item.computed_scroll_track.scale(scale_factor);
        let thumb_rect = self.element_data_mut().layout_item.computed_scroll_thumb.scale(scale_factor);

        let border_spec = BorderSpec::new(
            thumb_rect,
            [0.0, 0.0, 0.0, 0.0],
            scrollbar_thumb_radius,
            TrblRectangle::new_all(Color::TRANSPARENT),
        );
        let computed_border_spec = border_spec.compute_border_spec();

        renderer.draw_rect(track_rect, scrollbar_color.track_color);
        draw_borders_generic(renderer, &computed_border_spec, scrollbar_color.thumb_color, scale_factor);
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.current_style().overflow()[1] == Overflow::Scroll
    }

    /// Returns the element's [`ElementBox`] without any transforms applied.
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout_item.computed_box
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