use crate::geometry::{Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::layout::layout_item::LayoutItem;
use crate::palette;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::RenderList;
use crate::style::{Display, Style, Unit};
use crate::text::text_context::TextContext;
use taffy::{NodeId, Position, TaffyTree};

#[derive(Clone)]
pub(crate) struct Thumb {
    /// A pseudo thumb element, this is not stored in the user tree nor will it receive events.
    /// This is mostly for convenience, so that we can change the location and render it in the switch track container.
    pub(crate) layout_item: LayoutItem,
    /// The style of the thumb when the switch is toggled. This style will get merged with the default style + user style.
    pub(crate) thumb_style: Style,
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
            *style.border_radius_mut() =
                [(rounding, rounding), (rounding, rounding), (rounding, rounding), (rounding, rounding)];
        }

        style
    }

    pub(crate) fn default_toggled_thumb_style(&self, rounded: bool) -> Style {
        let style = Style::default();
        Style::merge(&self.default_thumb_style(rounded), &style)
    }

    pub(crate) fn thumb_style(&mut self, thumb_style: Style) {
        self.thumb_style = thumb_style;
    }

    pub(crate) fn toggled_thumb_style(&mut self, toggled_thumb_style: Style) {
        self.toggled_thumb_style = toggled_thumb_style;
    }

    pub(crate) fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _scale_factor: f64,
        toggled: bool,
        rounded: bool,
    ) -> NodeId {
        self.thumb_style = Style::merge(&self.default_thumb_style(rounded), &self.thumb_style);

        if toggled {
            self.thumb_style = Style::merge(&self.thumb_style, &self.default_toggled_thumb_style(rounded));
            self.thumb_style = Style::merge(&self.thumb_style, &self.toggled_thumb_style);
        }

        self.layout_item.build_tree(taffy_tree, self.thumb_style.to_taffy_style()).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(self.layout_item.taffy_node_id.unwrap()).unwrap();
        self.layout_item.resolve_box(position, transform, result, z_index, self.thumb_style.position());
        self.layout_item.finalize_borders(
            self.thumb_style.has_border(),
            self.thumb_style.border_radius(),
            self.thumb_style.border_color(),
        );
        self.layout_item.resolve_clip(clip_bounds);
    }

    pub(crate) fn draw(&mut self, renderer: &mut RenderList, scale_factor: f64) {
        if !self.thumb_style.visible() {
            return;
        }

        self.layout_item.draw_borders(renderer, &self.thumb_style, scale_factor);
    }
}
