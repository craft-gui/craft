use kurbo::Point;
use crate::elements::core::ElementInternals;
use crate::elements::Slider;
use crate::elements::slider::slider::SliderDirection;

impl Slider {
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
        let content_rectangle = self.computed_box().content_rectangle();
        let start = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.left() as f64
        } else {
            content_rectangle.top() as f64
        };
        let end = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.right() as f64
        } else {
            content_rectangle.bottom() as f64
        };

        let pointer_position_component =
            if self.get_direction() == SliderDirection::Horizontal { pointer_position.x } else { pointer_position.y };

        // [0, 1]
        let mut normalized_value = (pointer_position_component - start) / (end - start);
        normalized_value = normalized_value.clamp(0.0, 1.0);
        let mut value = normalized_value * self.get_max();

        // Round the value to the nearest step.
        value = (value / self.get_step()).round() * self.get_step();
        value = value.clamp(self.get_min(), self.get_max());

        value
    }

    pub(super) fn thumb_position(&self, thumb_value: f64) -> Point {
        let content_rectangle = self.computed_box().content_rectangle();

        let mut normalized_value = thumb_value / self.get_max();
        normalized_value = normalized_value.clamp(0.0, 1.0);

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
                content_rectangle.top() + value as f32 - thumb_offset,
                content_rectangle.top(),
                content_rectangle.bottom() - self.get_thumb_size() as f32,
            )
        };

        Point::new(x as f64, y as f64)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use super::*;
    use std::rc::Rc;

    fn make_test_slider() -> Rc<RefCell<Slider>> {
        let slider = Slider::new(16.0);
        slider
    }

    #[test]
    fn steps_one() {
        let slider_ref = make_test_slider();
        let mut slider = slider_ref.borrow_mut();
        
        slider.value(50.0);
        let next_step = slider.compute_step(1, slider.get_value());

        assert_eq!(next_step as i32, 51i32);
    }

    #[test]
    fn steps_down_one() {
        let slider_ref = make_test_slider();
        let mut slider = slider_ref.borrow_mut();
        
        slider.value(50.0);
        let next_step = slider.compute_step(-1, slider.get_value());

        assert_eq!(next_step as i32, 49i32);
    }
}
