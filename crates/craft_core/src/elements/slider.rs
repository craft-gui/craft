use crate::components::component::ComponentSpecification;
use crate::components::Event;
use crate::components::Props;
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::elements::thumb::Thumb;
use crate::events::CraftMessage;
use craft_primitives::geometry::borders::BorderSpec;
use craft_primitives::geometry::{Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use craft_renderer::renderer::RenderList;
use craft_renderer::renderer::Brush;
use crate::style::{Display, Style, Unit};
use crate::text::text_context::TextContext;
use crate::{generate_component_methods, palette};
use peniko::Color;
use std::any::Any;
use std::sync::Arc;
use kurbo::Affine;
use taffy::{NodeId, TaffyTree};
use ui_events::keyboard::{Code, KeyState};
use ui_events::keyboard::Code::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp};
use winit::window::Window;
use crate::elements::StatefulElement;
use smol_str::SmolStr;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct Slider {
    pub element_data: ElementData,
    pub step: f64,
    pub min: f64,
    pub max: f64,
    pub direction: SliderDirection,

    /// The color of the track to the left of the thumb. This may be disabled by setting this to `None`.
    value_track_color: Option<Color>,

    /// A pseudo thumb, this is not stored in the user tree nor will it receive events.
    /// This is mostly for convenience, so that we can change the location and render it in the slider track container.
    thumb: Thumb,
    rounded: bool,
}

#[derive(Clone, Copy, Default)]
pub struct SliderState {
    pub value: f64,
    pub dragging: bool,
}

impl StatefulElement<SliderState> for Slider {}

impl Element for Slider {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Slider"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        if !self.element_data.style.visible() {
            return;
        }

        self.draw_borders(renderer, element_state, scale_factor);

        // Draw the value track color to the left of the thumb.
        if let Some(value_track_color) = self.value_track_color {
            let element_data = self.element_data();
            let mut element_rect = self.computed_box_transformed();

            let borders = element_rect.border;
            let border_radius = element_data.current_style().border_radius();

            if self.direction == SliderDirection::Horizontal {
                element_rect.size.width = (self.thumb.layout_item.computed_box_transformed.position.x
                    - self.computed_box_transformed().position.x) as f32;

                // HACK: When the value track is visible add some extra width to make sure there are no gaps in the value track color.
                // The background track may show through on the left edge if the thumb is round.
                if element_rect.size.width > 0.0001 {
                    element_rect.size.width += self.thumb.size / 2.0;
                }
            } else {
                element_rect.size.height = (self.thumb.layout_item.computed_box_transformed.position.y
                    - self.computed_box_transformed().position.y) as f32;

                // HACK: When the value track is visible add some extra height to make sure there are no gaps in the value track color.
                // The background track may show through on the top edge if the thumb is round.
                if element_rect.size.height > 0.0001 {
                    element_rect.size.height += self.thumb.size / 2.0;
                }
            }

            let border_spec = BorderSpec::new(
                element_rect.border_rectangle(),
                [borders.top, borders.right, borders.bottom, borders.left],
                border_radius,
                element_data.current_style().border_color(),
            );
            let computed_border_spec = border_spec.compute_border_spec();
            let mut background_path = computed_border_spec.build_background_path();
            let scale_factor = Affine::scale(scale_factor);
            background_path.apply_affine(scale_factor);
            renderer.fill_bez_path(background_path, Brush::Color(value_track_color));
        }

        self.thumb.draw(renderer, scale_factor);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let child_node = self.thumb.compute_layout(taffy_tree, scale_factor, false, self.rounded);
        self.element_data.layout_item.push_child(&Some(child_node));
        
        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree(taffy_tree, style)
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let state = self.state(element_state);
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);
        self.finalize_borders(element_state);

        let thumb_position = self.thumb_position(state.value);

        self.thumb.finalize_layout(
            taffy_tree,
            thumb_position,
            z_index,
            transform,
            element_state,
            pointer,
            text_context,
            clip_bounds,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<&dyn Element>,
        _current_target: Option<&dyn Element>,
    ) {
        self.on_style_event(message, element_state, should_style, event);
        self.maybe_set_focus(message, event, target);
        let (state, base_state) = self.state_and_base_mut(element_state);
        let focused = base_state.focused;
        
        match message {
            CraftMessage::KeyboardInputEvent(key) => {
                if key.state != KeyState::Down || !focused {
                    return;
                }
                
                let new_value = match key.code {
                    ArrowUp | ArrowRight => Some(self.compute_step(1, state.value)),
                    ArrowDown | ArrowLeft => Some(self.compute_step(-1, state.value)),
                    Code::Home => Some(self.min),
                    Code::End => Some(self.max),
                    Code::PageUp => Some(self.compute_step(10, state.value)),
                    Code::PageDown => Some(self.compute_step(-10, state.value)),
                    _ => {
                        None
                    }
                };

                if let Some(new_value) = new_value {
                    state.value = new_value;
                    event.result_message(CraftMessage::SliderValueChanged(state.value));
                }
            }
            CraftMessage::PointerButtonUp(pointer_button_update) => {
                state.dragging = false;
                // FIXME: Turn pointer capture on with the correct device id.
                base_state.pointer_capture.remove(&DUMMY_DEVICE_ID);

                let value = self.compute_slider_value(&pointer_button_update.state.position);
                state.value = value;
                event.result_message(CraftMessage::SliderValueChanged(value));
            }
            CraftMessage::PointerButtonDown(pointer_button_update) => {
                state.dragging = true;
                // FIXME: Turn pointer capture on with the correct device id.
                base_state.pointer_capture.insert(DUMMY_DEVICE_ID, true);

                let value = self.compute_slider_value(&pointer_button_update.state.position);
                state.value = value;
                event.result_message(CraftMessage::SliderValueChanged(value));
            }
            CraftMessage::PointerMovedEvent(pointer_update) => {
                if !state.dragging {
                    return;
                }

                let value = self.compute_slider_value(&pointer_update.current.position);
                state.value = value;
                event.result_message(CraftMessage::SliderValueChanged(value));
            }
            _ => {}
        }
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(SliderState::default()),
        }
    }

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) {
        let (state, _base_state) = self.state_and_base_mut(element_state);
        let current_node_id = accesskit::NodeId(self.element_data().component_id);

        let mut current_node = accesskit::Node::new(accesskit::Role::Slider);
        current_node.set_value(*Box::new(state.value.to_string()));
        current_node.add_action(accesskit::Action::Click);
        current_node.add_action(accesskit::Action::Decrement);
        current_node.add_action(accesskit::Action::Increment);
        current_node.add_action(accesskit::Action::Focus);

        let padding_box = self.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });
        
        if let Some(parent_index) = parent_index {
            let parent_node = tree.nodes.get_mut(parent_index).unwrap();
            parent_node.1.push_child(current_node_id);
        }

        tree.nodes.push((current_node_id, current_node));
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();

        style.set_background(palette::css::LIGHT_GRAY);

        if self.direction == SliderDirection::Horizontal {
            style.set_width(Unit::Px(150.0));
            style.set_height(Unit::Px(10.0));
        } else {
            style.set_height(Unit::Px(150.0));
            style.set_width(Unit::Px(10.0));
        }

        style.set_display(Display::Block);

        if self.rounded {
            let rounding = self.thumb.size / 1.5;
            style.set_border_radius([
                (rounding, rounding),
                (rounding, rounding),
                (rounding, rounding),
                (rounding, rounding),
            ]);
        }

        style
    }
}

impl Slider {
    
    fn thumb_position(&self, thumb_value: f64) -> Point {
        let content_rectangle = self.computed_box().content_rectangle();

        let mut normalized_value = thumb_value / self.max;
        normalized_value = normalized_value.clamp(0.0, 1.0);

        let value = if self.direction == SliderDirection::Horizontal {
            normalized_value * content_rectangle.width as f64
        } else {
            normalized_value * content_rectangle.height as f64
        };

        let thumb_offset = self.thumb.size / 2.0;
        let x = if self.direction == SliderDirection::Horizontal {
            f32::clamp(
                content_rectangle.left() + value as f32 - thumb_offset,
                content_rectangle.left(),
                content_rectangle.right() - self.thumb.size,
            )
        } else {
            content_rectangle.left() - thumb_offset + content_rectangle.width / 2.0
        };

        let y = if self.direction == SliderDirection::Horizontal {
            content_rectangle.top() + content_rectangle.height / 2.0 - thumb_offset
        } else {
            f32::clamp(
                content_rectangle.top() + value as f32 - thumb_offset,
                content_rectangle.top(),
                content_rectangle.bottom() - self.thumb.size,
            )
        };

        Point::new(x as f64, y as f64)
    }

    /// Set the slider step value. Defaults to 1.
    pub fn step(mut self, value: f64) -> Self {
        self.step = value;
        self
    }

    /// Set the minimum slider value. Defaults to 0.
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Set the max slider value. Defaults to 100.
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Set the slider direction.
    pub fn direction(mut self, direction: SliderDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the thumb style.
    pub fn thumb_style(mut self, thumb_style: Style) -> Self {
        self.thumb.thumb_style(thumb_style);
        self
    }

    /// Enable rounding in the thumb and track.
    pub fn round(mut self) -> Self {
        self.rounded = true;
        self
    }

    /// The color of the track to the left of the thumb. This may be disabled by setting this to `None`.
    pub fn value_track_color(mut self, color: Option<Color>) -> Self {
        self.value_track_color = color;
        self
    }

    fn compute_step(&self, by: i32, current_value: f64) -> f64 {
        let delta = by.abs() as f64 * self.step;

        let value = if by > 0 {
            current_value + delta
        } else {
            current_value - delta
        };

        value.clamp(self.min, self.max)
    }

    fn compute_slider_value(&self, pointer_position: &Point) -> f64 {
        let content_rectangle = self.computed_box().content_rectangle();
        let start = if self.direction == SliderDirection::Horizontal {
            content_rectangle.left() as f64
        } else {
            content_rectangle.top() as f64
        };
        let end = if self.direction == SliderDirection::Horizontal {
            content_rectangle.right() as f64
        } else {
            content_rectangle.bottom() as f64
        };

        let pointer_position_component =
            if self.direction == SliderDirection::Horizontal { pointer_position.x } else { pointer_position.y };

        // [0, 1]
        let mut normalized_value = (pointer_position_component - start) / (end - start);
        normalized_value = normalized_value.clamp(0.0, 1.0);
        let mut value = normalized_value * self.max;

        // Round the value to the nearest step.
        value = (value / self.step).round() * self.step;
        value = value.clamp(self.min, self.max);

        value
    }

    pub fn new(thumb_size: f32) -> Slider {
        let mut thumb = Thumb {
            layout_item: Default::default(),
            thumb_style: Default::default(),
            toggled_thumb_style: Default::default(),
            size: thumb_size,
        };
        let mut style = Style::default();
        style.set_background(palette::css::DODGER_BLUE);
        thumb.thumb_style(style);

        Slider {
            element_data: Default::default(),
            step: 1.0,
            min: 0.0,
            max: 100.0,
            direction: Default::default(),
            value_track_color: Some(palette::css::DODGER_BLUE),
            thumb,
            rounded: false,
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Slider {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
