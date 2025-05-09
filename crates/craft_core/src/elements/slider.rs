use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::layout::layout_context::LayoutContext;
use crate::elements::thumb::Thumb;
use crate::events::CraftMessage;
use crate::geometry::borders::BorderSpec;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::renderer::Brush;
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods, palette};
use peniko::Color;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::event::ElementState;
use winit::window::Window;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
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
        text_context: &mut TextContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        
        self.draw_borders(renderer);

        // Draw the value track color to the left of the thumb.
        if let Some(value_track_color) = self.value_track_color {
            let element_data = self.element_data();
            let mut element_rect = self.element_data().computed_box_transformed;

            let borders = element_rect.border;
            let border_radius = element_data.current_style().border_radius();

            if self.direction == SliderDirection::Horizontal {
                element_rect.size.width = self.thumb.pseudo_thumb.element_data.computed_box_transformed.position.x - self.element_data().computed_box_transformed.position.x;

                // HACK: When the value track is visible add some extra width to make sure there are no gaps in the value track color.
                // The background track may show through on the left edge if the thumb is round.
                if element_rect.size.width > 0.0001 {
                    element_rect.size.width += self.thumb.size / 2.0;
                }
            } else {
                element_rect.size.height = self.thumb.pseudo_thumb.element_data.computed_box_transformed.position.y - self.element_data().computed_box_transformed.position.y;

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
            let background_path = computed_border_spec.build_background_path();
            renderer.fill_bez_path(background_path, Brush::Color(value_track_color));
        }

        self.thumb.pseudo_thumb.draw(renderer, text_context, taffy_tree, _root_node, element_state, pointer, window);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let child_node = self.thumb.compute_layout(taffy_tree, element_state, scale_factor, false, self.rounded);

        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);
        self.element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &[child_node]).unwrap());
        self.element_data().taffy_node_id
    }
    
    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
    ) {
        let state = self.get_state(element_state);
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.finalize_borders();

        let thumb_position = self.thumb_position(state.value);

        self.thumb.finalize_layout(
            taffy_tree,
            thumb_position,
            z_index,
            transform,
            element_state,
            pointer,
            text_context,
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
    ) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<SliderState>().unwrap();

        if let CraftMessage::PointerButtonEvent(pointer) = message {

            if pointer.state == ElementState::Pressed {
                state.dragging = true;
                // FIXME: Turn pointer capture on with the correct device id.
                base_state.base.pointer_capture.insert(DUMMY_DEVICE_ID, true);
            } else if pointer.state == ElementState::Released {
                state.dragging = false;
                // FIXME: Turn pointer capture on with the correct device id.
                base_state.base.pointer_capture.remove(&DUMMY_DEVICE_ID);
            }

            let value = self.compute_slider_value(&pointer.position);
            state.value = value;
            return UpdateResult::default().result_message(CraftMessage::SliderValueChanged(value));
        }

        if let CraftMessage::PointerMovedEvent(pointer) = message {
            if !state.dragging {
                return UpdateResult::default();
            }

            let value = self.compute_slider_value(&pointer.position);
            state.value = value;
            return UpdateResult::default().result_message(CraftMessage::SliderValueChanged(value));
        }

        UpdateResult::default()
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(SliderState::default()),
        }
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.background_mut() = palette::css::LIGHT_GRAY;
        if self.direction == SliderDirection::Horizontal {
            *style.width_mut() = Unit::Px(150.0);
            *style.height_mut() = Unit::Px(10.0);
        } else {
            *style.height_mut() = Unit::Px(150.0);
            *style.width_mut() = Unit::Px(10.0);
        }
        
        *style.display_mut() = Display::Block;

        if self.rounded {
            let rounding = self.thumb.size / 1.5;
            *style.border_radius_mut() = [(rounding, rounding), (rounding, rounding), (rounding, rounding), (rounding, rounding)];
        }


        style
    }
}

impl Slider {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a SliderState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn thumb_position(&self, thumb_value: f64) -> Point {
        let content_rectangle = self.element_data.computed_box.content_rectangle();
        
        let mut normalized_value = thumb_value / self.max;
        normalized_value = normalized_value.clamp(0.0, 1.0);
        
        let value = if self.direction == SliderDirection::Horizontal {
            normalized_value * content_rectangle.width as f64
        } else {
            normalized_value * content_rectangle.height as f64
        };
        
        let thumb_offset = self.thumb.size / 2.0;
        let x = if self.direction == SliderDirection::Horizontal {
            f32::clamp(content_rectangle.left() + value as f32 - thumb_offset, content_rectangle.left(), content_rectangle.right() - self.thumb.size)
        } else {
            content_rectangle.left() - thumb_offset + content_rectangle.width / 2.0
        };
        
        let y = if self.direction == SliderDirection::Horizontal {
            content_rectangle.top() +
                content_rectangle.height / 2.0 - thumb_offset
        } else {
            f32::clamp(content_rectangle.top() + value as f32 - thumb_offset, content_rectangle.top(), content_rectangle.bottom() - self.thumb.size)
        };
        
        Point::new(x, y)
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

    fn compute_slider_value(&self, pointer_position: &Point) -> f64 {
        let content_rectangle = self.element_data.computed_box.content_rectangle();
        let start = if self.direction == SliderDirection::Horizontal { content_rectangle.left() as f64 } else { content_rectangle.top() as f64 };
        let end = if self.direction == SliderDirection::Horizontal { content_rectangle.right() as f64 } else { content_rectangle.bottom() as f64 };

        let pointer_position_component = if self.direction == SliderDirection::Horizontal { pointer_position.x } else { pointer_position.y };

        // [0, 1]
        let mut normalized_value = (pointer_position_component as f64 - start) / (end - start);
        normalized_value = normalized_value.clamp(0.0, 1.0);
        let mut value = normalized_value * self.max;

        // Round the value to the nearest step.
        value = (value / self.step).round() * self.step;
        value = value.clamp(self.min, self.max);

        value
    }

    pub fn new(thumb_size: f32) -> Slider {
        let mut thumb = Thumb {
            pseudo_thumb: Default::default(),
            toggled_thumb_style: Default::default(),
            size: thumb_size,
        };
        let mut style = Style::default();
        *style.background_mut() = palette::css::DODGER_BLUE;
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
