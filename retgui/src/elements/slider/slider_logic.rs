use retgui_primitives::geometry::Point;

use crate::elements::slider::slider_element::SliderDirection;
use crate::elements::{ElementNodeData, SliderNode};

impl SliderNode {
    pub(super) fn compute_step(&self, by: i32, current_value: f64) -> f64 {
        let delta = by.abs() as f64 * self.get_step();

        let value = if by > 0 {
            current_value + delta
        } else {
            current_value - delta
        };

        value.clamp(self.get_min(), self.get_max())
    }

    pub(super) fn compute_slider_value(&self, pointer_position: &Point) -> f64 {
        let content_rectangle = self.element_data().layout.world_box().content_rectangle();
        let start = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.left() as f64
        } else {
            content_rectangle.bottom() as f64
        };
        let end = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.right() as f64
        } else {
            content_rectangle.top() as f64
        };

        let pointer_position_component = if self.get_direction() == SliderDirection::Horizontal {
            pointer_position.x
        } else {
            pointer_position.y
        };

        let track_length = end - start;
        if track_length == 0.0 {
            return self.get_min();
        }

        // [0, 1]
        let normalized_value = ((pointer_position_component - start) / track_length).clamp(0.0, 1.0);
        let range = self.get_max() - self.get_min();
        let raw = self.get_min() + normalized_value * range;

        // Round the value to the nearest step.
        let stepped = self.get_min() + ((raw - self.get_min()) / self.get_step()).round() * self.get_step();
        stepped.clamp(self.get_min(), self.get_max())
    }

    pub(super) fn thumb_position(&self, thumb_value: f64) -> Point {
        self.thumb_position_in_rect(thumb_value, self.element_data().layout.world_box().content_rectangle())
    }

    pub(super) fn local_thumb_position(&self, thumb_value: f64) -> Point {
        self.thumb_position_in_rect(thumb_value, self.element_data().layout.computed_box.content_rectangle())
    }

    fn thumb_position_in_rect(
        &self,
        thumb_value: f64,
        content_rectangle: retgui_primitives::geometry::Rectangle,
    ) -> Point {
        let range = self.get_max() - self.get_min();
        let normalized_value = if range == 0.0 {
            0.0
        } else {
            ((thumb_value - self.get_min()) / range).clamp(0.0, 1.0)
        };

        let value = if self.get_direction() == SliderDirection::Horizontal {
            normalized_value * content_rectangle.width as f64
        } else {
            normalized_value * content_rectangle.height as f64
        };

        let thumb_offset = self.get_thumb_size() as f32 / 2.0;
        let x = if self.get_direction() == SliderDirection::Horizontal {
            f32::clamp(
                content_rectangle.left() + value as f32 - thumb_offset,
                content_rectangle.left(),
                content_rectangle.right() - self.get_thumb_size() as f32,
            )
        } else {
            content_rectangle.left() - thumb_offset + content_rectangle.width / 2.0
        };

        let y = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.top() + content_rectangle.height / 2.0 - thumb_offset
        } else {
            f32::clamp(
                content_rectangle.bottom() - value as f32 - thumb_offset,
                content_rectangle.top(),
                content_rectangle.bottom() - self.get_thumb_size() as f32,
            )
        };

        Point::new(x as f64, y as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{Elements, Slider};

    #[test]
    fn steps_one() {
        let mut elements = Elements::new();
        let slider = Slider::new(&mut elements, 16.0);
        let slider = elements.get_as_mut::<SliderNode>(slider.inner);

        slider.set_value(50.0);
        let next_step = slider.compute_step(1, slider.get_value());

        assert_eq!(next_step as i32, 51i32);
    }

    #[test]
    fn steps_down_one() {
        let mut elements = Elements::new();
        let slider = Slider::new(&mut elements, 16.0);
        let slider = elements.get_as_mut::<SliderNode>(slider.inner);

        slider.set_value(50.0);
        let next_step = slider.compute_step(-1, slider.get_value());

        assert_eq!(next_step as i32, 49i32);
    }
}
