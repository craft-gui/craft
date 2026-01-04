use craft_primitives::geometry::Rectangle;
use craft_primitives::geometry::borders::CssRoundedRect;
use craft_renderer::RenderList;
use kurbo::Vec2;

use crate::elements::slider::slider::SliderDirection;
use crate::elements::{ElementImpl, SliderInner};
use crate::layout::layout_item::{CssComputedBorder, draw_borders_generic};

fn border_radius_to_vec_radius(border_radius: [(f32, f32); 4]) -> [Vec2; 4] {
    let br = border_radius;
    [
        Vec2::new(br[0].0 as f64, br[0].1 as f64),
        Vec2::new(br[1].0 as f64, br[1].1 as f64),
        Vec2::new(br[2].0 as f64, br[2].1 as f64),
        Vec2::new(br[3].0 as f64, br[3].1 as f64),
    ]
}

impl SliderInner {
    pub(super) fn draw_track(&mut self, renderer: &mut RenderList, scale_factor: f64) {
        if let Some(track_color) = self.get_track_color() {
            let mut element_rect = self.get_computed_box_transformed();
            let thumb_pos = self.thumb_position(self.get_value());

            if self.get_direction() == SliderDirection::Horizontal {
                element_rect.size.width = (thumb_pos.x - self.get_computed_box_transformed().position.x) as f32;

                // HACK: When the value track is visible add some extra width to make sure there are no gaps in the value track color.
                // The background track may show through on the left edge if the thumb is round.
                if element_rect.size.width > 0.0001 {
                    element_rect.size.width += self.get_thumb_size() as f32 / 2.0;
                }
            } else {
                element_rect.size.height = (thumb_pos.y - self.get_computed_box_transformed().position.y) as f32;

                // HACK: When the value track is visible add some extra height to make sure there are no gaps in the value track color.
                // The background track may show through on the top edge if the thumb is round.
                if element_rect.size.height > 0.0001 {
                    element_rect.size.height += self.get_thumb_size() as f32 / 2.0;
                }
            }

            // Use the specified border radius or default to the slider's border radius.
            let thumb_radii = if let Some(br) = self.get_track_border_radius() {
                border_radius_to_vec_radius(br)
            } else {
                border_radius_to_vec_radius(self.style().border_radius())
            };

            let css_rounded_rect = CssRoundedRect::new(
                element_rect.border_rectangle().to_kurbo(),
                [0.0, 0.0, 0.0, 0.0],
                thumb_radii,
            );
            let mut computed_border_spec = CssComputedBorder::new(css_rounded_rect);
            computed_border_spec.scale(scale_factor);

            let color_rect = [track_color, track_color, track_color, track_color];
            draw_borders_generic(renderer, &computed_border_spec, color_rect, track_color);
        }
    }

    pub(super) fn draw_thumb(&mut self, renderer: &mut RenderList) {
        let thumb_pos = self.thumb_position(self.get_value());
        let thumb_size = self.get_thumb_size();
        let thumb_background_color = self.get_thumb_color();
        let thumb_rect = Rectangle::new(
            thumb_pos.x as f32,
            thumb_pos.y as f32,
            thumb_size as f32,
            thumb_size as f32,
        );

        // Use the specified border radius or default to 50% (a circle).
        let thumb_radii = if let Some(br) = self.get_thumb_border_radius() {
            border_radius_to_vec_radius(br)
        } else {
            let half_size = thumb_size / 2.0;
            let half_size = Vec2::new(half_size, half_size);
            [half_size, half_size, half_size, half_size]
        };

        let css_rounded_rect = CssRoundedRect::new(thumb_rect.to_kurbo(), [0.0, 0.0, 0.0, 0.0], thumb_radii);
        let computed_border_spec = CssComputedBorder::new(css_rounded_rect);
        let color_rect = [
            thumb_background_color,
            thumb_background_color,
            thumb_background_color,
            thumb_background_color,
        ];
        draw_borders_generic(renderer, &computed_border_spec, color_rect, thumb_background_color);
    }
}
