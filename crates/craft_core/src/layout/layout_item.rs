use kurbo::Affine;
use crate::geometry::borders::{BorderSpec, ComputedBorderSpec};
use crate::geometry::side::Side;
use crate::geometry::PointConverter;
use crate::geometry::{Border, ElementBox, Margin, Padding, Point, Rectangle, Size, TrblRectangle};
use crate::layout::layout_context::LayoutContext;
use crate::renderer::{Brush, RenderList};
use crate::style::Style;
use peniko::Color;
use taffy::{NodeId, Position, TaffyTree};

#[derive(Clone, Default)]
pub struct LayoutItem {
    /// The taffy node id after this element is laid out.
    /// This may be None if this is a non-visual element like Font.
    pub taffy_node_id: Option<NodeId>,

    pub content_size: Size<f32>,
    // The computed values after transforms are applied.
    pub computed_box_transformed: ElementBox,
    // The computed values without any transforms applied to them.
    pub computed_box: ElementBox,
    pub computed_scrollbar_size: Size<f32>,
    pub scrollbar_size: Size<f32>,
    pub computed_scroll_track: Rectangle,
    pub computed_scroll_thumb: Rectangle,
    pub computed_border: ComputedBorderSpec,
    pub(crate) max_scroll_y: f32,

    pub layout_order: u32,
    pub clip_bounds: Option<Rectangle>,

    //  ---
    pub child_nodes: Vec<NodeId>,
}

impl LayoutItem {
    pub fn push_child(&mut self, child: &Option<NodeId>) {
        if let Some(taffy_node_id) = child.as_ref() {
            self.child_nodes.push(*taffy_node_id);
        }
    }

    pub fn build_tree(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, style: taffy::Style) -> Option<NodeId> {
        self.taffy_node_id = Some(taffy_tree.new_with_children(style, &self.child_nodes).unwrap());
        self.taffy_node_id.clone()
    }

    pub fn build_tree_with_context(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        style: taffy::Style,
        layout_context: LayoutContext,
    ) -> Option<NodeId> {
        self.taffy_node_id = Some(taffy_tree.new_leaf_with_context(style, layout_context).unwrap());
        self.taffy_node_id.clone()
    }

    pub fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: Affine,
        result: &taffy::Layout,
        layout_order: &mut u32,
        position: Position,
    ) {
        self.layout_order = *layout_order;
        *layout_order += 1;

        let at_position = match position {
            Position::Relative => relative_position + Point::from_taffy_point(result.location).to_vec2(),
            // We'll need to create our own enum for this because currently, relative acts more like static and absolute acts like relative.
            Position::Absolute => relative_position + Point::from_taffy_point(result.location).to_vec2(),
        };

        let mut size = result.size.into();
        // FIXME: Don't use the content size for position absolute containers.
        // The following is a broken layout using result.size.
        // └──  FLEX COL [x: 1    y: 44   w: 140  h: 45   content_w: 139  content_h: 142  border: l:1 r:1 t:1 b:1, padding: l:12 r:12 t:8 b:8] (NodeId(4294967303))
        //     ├──  LEAF [x: 13   y: 9    w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967298))
        //     ├──  LEAF [x: 13   y: 34   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967299))
        //     ├──  LEAF [x: 13   y: 59   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967300))
        //     ├──  LEAF [x: 13   y: 84   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967301))
        //     └──  LEAF [x: 13   y: 109  w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967302))
        if position == Position::Absolute {
            size = Size::new(
                f32::max(result.size.width, result.content_size.width),
                f32::max(result.size.height, result.content_size.height),
            );
        }

        self.content_size = Size::new(result.content_size.width, result.content_size.height);
        self.computed_box = ElementBox {
            margin: Margin::new(result.margin.top, result.margin.right, result.margin.bottom, result.margin.left),
            border: Border::new(result.border.top, result.border.right, result.border.bottom, result.border.left),
            padding: Padding::new(result.padding.top, result.padding.right, result.padding.bottom, result.padding.left),
            position: at_position,
            size,
        };
        self.computed_box_transformed = self.computed_box.transform(scroll_transform);
    }

    pub fn finalize_borders(
        &mut self,
        has_border: bool,
        border_radius: [(f32, f32); 4],
        border_color: TrblRectangle<Color>,
    ) {
        // OPTIMIZATION: Don't compute the border if no border style values have been modified.
        if !has_border {
            return;
        }

        let element_rect = self.computed_box_transformed;
        let borders = element_rect.border;
        let border_spec = BorderSpec::new(
            element_rect.border_rectangle(),
            [borders.top, borders.right, borders.bottom, borders.left],
            border_radius,
            border_color,
        );
        self.computed_border = border_spec.compute_border_spec();
    }

    pub fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.clip_bounds = clip_bounds;
    }

    pub fn draw_borders(&self, renderer: &mut RenderList, current_style: &Style, scale_factor: f64) {
        let background_color = current_style.background();

        // OPTIMIZATION: Draw a normal rectangle if no border values have been modified.
        if !current_style.has_border() {
            renderer.draw_rect(self.computed_box_transformed.padding_rectangle().scale(scale_factor), background_color);
            return;
        }
        
        let computed_border_spec = &self.computed_border;
        draw_borders_generic(renderer, computed_border_spec, background_color, scale_factor);
    }
}

pub(crate) fn draw_borders_generic(renderer: &mut RenderList, computed_border_spec: &ComputedBorderSpec, bg_color: Color, scale_factor: f64) {
    let background_color = bg_color;
    let scale_factor = Affine::scale(scale_factor);

    let mut background_path = computed_border_spec.build_background_path();
    background_path.apply_affine(scale_factor);

    renderer.fill_bez_path(background_path, Brush::Color(background_color));

    let top = computed_border_spec.get_side(Side::Top);
    let right = computed_border_spec.get_side(Side::Right);
    let bottom = computed_border_spec.get_side(Side::Bottom);
    let left = computed_border_spec.get_side(Side::Left);

    let mut border_top_path = computed_border_spec.build_side_path(Side::Top);
    let mut border_right_path = computed_border_spec.build_side_path(Side::Right);
    let mut border_bottom_path = computed_border_spec.build_side_path(Side::Bottom);
    let mut border_left_path = computed_border_spec.build_side_path(Side::Left);

    border_top_path.apply_affine(scale_factor);
    border_right_path.apply_affine(scale_factor);
    border_bottom_path.apply_affine(scale_factor);
    border_left_path.apply_affine(scale_factor);

    renderer.fill_bez_path(border_top_path, Brush::Color(top.color));
    renderer.fill_bez_path(border_right_path, Brush::Color(right.color));
    renderer.fill_bez_path(border_bottom_path, Brush::Color(bottom.color));
    renderer.fill_bez_path(border_left_path, Brush::Color(left.color));
}