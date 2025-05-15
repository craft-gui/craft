use taffy::{NodeId, Position, TaffyTree};
use crate::elements::Container;
use crate::elements::element::Element;
use crate::layout::layout_context::LayoutContext;
use crate::geometry::Point;
use crate::palette;
use crate::reactive::element_state_store::ElementStateStore;
use crate::style::{Display, Style, Unit};
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub(crate) struct Thumb {
    /// A pseudo thumb element, this is not stored in the user tree nor will it receive events.
    /// This is mostly for convenience, so that we can change the location and render it in the switch track container.
    pub(crate) pseudo_thumb: Container,
    /// The style of the thumb when the switch is toggled. This style will get merged with the default style + user style.
    pub(crate) toggled_thumb_style: Style,
    /// The size of the thumb in pixels.
    pub(crate) size: f32,
}

impl Thumb {
    pub(crate) fn default_thumb_style(&self, rounded: bool) -> Style {
        let mut style = Style::default();

        *style.display_mut() = Display::Block;
        *style.width_mut() = Unit::Px(self.size);
        *style.height_mut() = Unit::Px(self.size);
        *style.background_mut() = palette::css::WHITE;
        *style.position_mut() = Position::Relative;

        if rounded {
            let rounding = self.size / 2.0;
            *style.border_radius_mut() = [(rounding, rounding), (rounding, rounding), (rounding, rounding), (rounding, rounding)];
        }

        style
    }

    pub(crate) fn default_toggled_thumb_style(&self, rounded: bool) -> Style {
        let style = Style::default();
        Style::merge(&self.default_thumb_style(rounded), &style)
    }

    pub(crate) fn thumb_style(&mut self, thumb_style: Style) {
        self.pseudo_thumb.element_data_mut().style = thumb_style;
    }

    pub(crate) fn toggled_thumb_style(&mut self, toggled_thumb_style: Style) {
        self.toggled_thumb_style = toggled_thumb_style;
    }

    pub(crate) fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>,
                          element_state: &mut ElementStateStore,
                          scale_factor: f64, toggled: bool, rounded: bool) -> NodeId {
        self.pseudo_thumb.element_data_mut().style =
            Style::merge(&self.default_thumb_style(rounded), &self.pseudo_thumb.element_data_mut().style);

        if toggled {
            self.pseudo_thumb.element_data_mut().style =
                Style::merge(&self.pseudo_thumb.element_data().style, &self.default_toggled_thumb_style(rounded));
            self.pseudo_thumb.element_data_mut().style =
                Style::merge(&self.pseudo_thumb.element_data().style, &self.toggled_thumb_style);
        }

        self.pseudo_thumb.compute_layout(taffy_tree, element_state, scale_factor).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finalize_layout(&mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
    ) {
        self.pseudo_thumb.finalize_layout(
            taffy_tree,
            self.pseudo_thumb.element_data.taffy_node_id.unwrap(),
            position,
            z_index,
            transform,
            element_state,
            pointer,
            text_context,
        );
    }
}