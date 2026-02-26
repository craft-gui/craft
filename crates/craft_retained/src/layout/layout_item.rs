use craft_primitives::geometry::borders::{BOTTOM, CssRoundedRect, LEFT, RIGHT, TOP};
use craft_primitives::geometry::{Border, ElementBox, Margin, Padding, Point, Rectangle, Size, TrblRectangle};
use craft_renderer::{Brush, RenderList};
use kurbo::{Affine, BezPath, Shape, Vec2};
use peniko::Color;
use taffy::NodeId;
use craft_renderer::renderer::BoxShadowCmd;
use crate::style::{BoxShadow, Position, Style};

impl CssComputedBorder {
    pub(crate) fn scale(&mut self, scale_factor: f64) {
        let scale_factor = Affine::scale(scale_factor);

        self.background.apply_affine(scale_factor);

        for side in self.sides.iter_mut().flatten() {
            side.apply_affine(scale_factor);
        }
    }
}

impl CssComputedBorder {
    pub(crate) fn new(css_rect: CssRoundedRect) -> Self {
        let top = css_rect.get_side(TOP);
        let right = css_rect.get_side(RIGHT);
        let bottom = css_rect.get_side(BOTTOM);
        let left = css_rect.get_side(LEFT);
        let background = css_rect.to_path(0.1f64);

        Self {
            css_rect,
            sides: [top, right, bottom, left],
            background,
        }
    }
}

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
    pub computed_border_sides: Option<[BezPath; 4]>,
    pub(crate) max_scroll_y: f32,

    pub layout_order: u32,
    pub clip_bounds: Option<Rectangle>,

    //cache_border_spec: Option<(CssRoundedRect, f64)>, // f64 for scale factor
    cache_border_spec: Option<BorderSpec>,
    cache_box_shadows: Option<ComputedBoxShadows>,
    computed_border: ComputedBorder,
    /// True if the layout is new.
    pub has_new_layout: bool,
    transform: Affine,
    pub position: Point,
}

impl LayoutItem {
    pub(crate) fn get_transform(&self) -> Affine {
        self.transform
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
            Position::Relative => relative_position + from_taffy_point(result.location).to_vec2(),
            // We'll need to create our own enum for this because currently, relative acts more like static and absolute acts like relative.
            Position::Absolute => relative_position + from_taffy_point(result.location).to_vec2(),
        };

        let mut size = Size {
            width: result.size.width,
            height: result.size.height,
        };
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
            margin: Margin::new(
                result.margin.top,
                result.margin.right,
                result.margin.bottom,
                result.margin.left,
            ),
            border: Border::new(
                result.border.top,
                result.border.right,
                result.border.bottom,
                result.border.left,
            ),
            padding: Padding::new(
                result.padding.top,
                result.padding.right,
                result.padding.bottom,
                result.padding.left,
            ),
            position: at_position,
            size,
        };
        self.computed_box_transformed = self.computed_box.transform(scroll_transform);
        self.transform = scroll_transform;
        self.position = relative_position;
    }

    pub fn apply_borders(
        &mut self,
        has_border: bool,
        border_radius: [(f32, f32); 4],
        scale_factor: f64,
        border_color: &TrblRectangle<Color>,
        box_shadows: Vec<BoxShadow>,
    ) {
        let element_rect = self.computed_box_transformed;
        let border_spec = BorderSpec {
            rect: element_rect.border_rectangle(),
            width: element_rect.border,
            radii: border_radius,
            scale_factor,
            box_shadows,
        };

        if Some(&border_spec) == self.cache_border_spec.as_ref() {
            return;
        }
        self.cache_border_spec = Some(border_spec);

        let is_rectangle = border_radius[0] == (0.0, 0.0)
            && border_radius[1] == (0.0, 0.0)
            && border_radius[2] == (0.0, 0.0)
            && border_radius[3] == (0.0, 0.0);

        // OPTIMIZATION: Don't compute the border if no border style values have been modified.
        // Note: even if all radii are 0.0, if the color varies between two edges,
        // then the color will split diagonally and cannot be drawn as a rect.
        if !has_border || (is_rectangle && border_color.are_edges_uniform()) {
            self.computed_border = ComputedBorder::Simple;
            self.apply_box_shadows(scale_factor, border_radius);
            return;
        }

        let borders = element_rect.border;
        let border_spec = CssRoundedRect::new(
            element_rect.border_rectangle().to_kurbo(),
            [
                borders.top as f64,
                borders.right as f64,
                borders.bottom as f64,
                borders.left as f64,
            ],
            border_radius.map(|radii| Vec2::new(radii.0 as f64, radii.1 as f64)),
        );

        let mut computed = CssComputedBorder::new(border_spec);
        computed.scale(scale_factor);
        self.computed_border = ComputedBorder::CssComputedBorder(computed);

        self.apply_box_shadows(scale_factor, border_radius);
    }

    fn apply_box_shadows(&mut self, scale_factor: f64, border_radius: [(f32, f32); 4],) {
        let scale_transform = Affine::scale(scale_factor);
        let box_shadows = &self.cache_border_spec.as_ref().unwrap().box_shadows;
        match &self.computed_border {
            ComputedBorder::CssComputedBorder(css_rect) => {
                let mut outline = css_rect.css_rect.get_outline();
                outline.apply_affine(scale_transform);
                let mut cache_box_shadows = ComputedBoxShadows {
                    outline: BezPathOrRect::BezPath(outline),
                    box_shadows: Vec::with_capacity(box_shadows.len()),
                    border_box: self.computed_box_transformed.border_rectangle().scale(scale_factor),
                };
                for box_shadow in box_shadows {
                    let offset = Vec2::new(box_shadow.offset_x, box_shadow.offset_y);


                    let element_rect = self.computed_box_transformed;
                    let borders = element_rect.border;
                    let inset_css_rect = CssRoundedRect::new(
                        element_rect.border_rectangle().to_kurbo().inflate(-box_shadow.spread_radius, -box_shadow.spread_radius),
                        [
                            borders.top as f64,
                            borders.right as f64,
                            borders.bottom as f64,
                            borders.left as f64,
                        ],
                        border_radius.map(|radii| Vec2::new((radii.0 as f64 - box_shadow.spread_radius).max(0.0), (radii.1 as f64 - box_shadow.spread_radius).max(0.0))),
                    );
                    let mut inset_rect_outline = inset_css_rect.get_outline_with_radius(0.0);
                    inset_rect_outline.apply_affine(Affine::translate(offset));
                    inset_rect_outline.apply_affine(scale_transform);
                    cache_box_shadows.box_shadows.push(ComputedBoxShadow {
                        inset: box_shadow.inset,
                        shape: BezPathOrRect::BezPath(inset_rect_outline),
                        offset: offset * scale_factor,
                        blur_radius: box_shadow.blur_radius * scale_factor,
                        color: box_shadow.color,
                    })
                }
                self.cache_box_shadows = Some(cache_box_shadows);
            }
            ComputedBorder::Simple => {
                let outline = self.computed_box_transformed.border_rectangle();

                let mut cache_box_shadows = ComputedBoxShadows {
                    outline: BezPathOrRect::Rect(outline),
                    box_shadows: Vec::with_capacity(box_shadows.len()),
                    border_box: self.computed_box_transformed.border_rectangle().scale(scale_factor),
                };
                for box_shadow in box_shadows {
                    let radius_modifier = if box_shadow.inset {-1.0} else {1.0};
                    let offset = Vec2::new(box_shadow.offset_x, box_shadow.offset_y);
                    cache_box_shadows.box_shadows.push(ComputedBoxShadow {
                        inset: box_shadow.inset,
                        shape: BezPathOrRect::Rect(outline.expand((box_shadow.spread_radius * radius_modifier) as f32).scale(scale_factor)),
                        offset: offset * scale_factor,
                        blur_radius: box_shadow.blur_radius * scale_factor,
                        color: box_shadow.color,
                    })
                }
                self.cache_box_shadows = Some(cache_box_shadows);
            }
            ComputedBorder::None => {}
        }
    }

    pub fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.clip_bounds = clip_bounds;
    }

    pub fn draw_borders(&self, renderer: &mut RenderList, current_style: &Style, scale_factor: f64) {
        if let Some(cache_box_shadows) = &self.cache_box_shadows {
            for shadow in &cache_box_shadows.box_shadows {
                if shadow.inset {
                    continue;
                }

                renderer.draw_outset_box_shadow(BoxShadowCmd {
                    inset: false,
                    offset: shadow.offset,
                    outline: cache_box_shadows.outline.to_path(),
                    path: shadow.shape.to_path(),
                    blur_radius: shadow.blur_radius,
                    color: shadow.color,
                    border_box: cache_box_shadows.border_box,
                });
            }
        }

        let background_color = current_style.get_background_color();

        // OPTIMIZATION: Draw a normal rectangle if no border values have been modified.
        match &self.computed_border {
            ComputedBorder::None => {}
            ComputedBorder::Simple => {
                let padding_rect = self.computed_box_transformed.padding_rectangle().scale(scale_factor);
                let border_rect = self.computed_box_transformed.border_rectangle();
                // Draw the background.
                if background_color.components[3] != 0.0 {
                    renderer.draw_rect(padding_rect, background_color);
                }
                let thickness = self.cache_border_spec.as_ref().unwrap().width.top;
                let border_color = current_style.get_border_color().top;
                if thickness != 0.0 && border_color.components[3] != 0.0 {
                    renderer.draw_rect_outline(border_rect, border_color, thickness as f64);
                }
            }
            ComputedBorder::CssComputedBorder(computed_border) => {
                draw_borders_generic(
                    renderer,
                    computed_border,
                    current_style.get_border_color().to_array(),
                    background_color,
                );
            }
        }

        if let Some(cache_box_shadows) = &self.cache_box_shadows {
            for shadow in &cache_box_shadows.box_shadows {
                if !shadow.inset {
                    continue;
                }

                renderer.draw_outset_box_shadow(BoxShadowCmd {
                    inset: true,
                    offset: shadow.offset,
                    outline: cache_box_shadows.outline.to_path(),
                    path: shadow.shape.to_path(),
                    blur_radius: shadow.blur_radius,
                    color: shadow.color,
                    border_box: cache_box_shadows.border_box,
                });
            }
        }
    }
}

pub(crate) fn draw_borders_generic(
    renderer: &mut RenderList,
    computed_border: &CssComputedBorder,
    side_colors: [Color; 4],
    bg_color: Color,
) {
    let background_color = bg_color;

    if background_color.components[3] != 0.0 {
        let background_path = computed_border.background.clone();
        renderer.fill_bez_path(background_path, Brush::Color(background_color));
    }

    for (side_index, side) in computed_border.sides.iter().enumerate() {
        if let Some(side) = side {
            let side = side.clone();
            renderer.fill_bez_path(side, Brush::Color(side_colors[side_index]));
        }
    }
}

#[derive(Clone)]
enum BezPathOrRect {
    Rect(Rectangle),
    BezPath(BezPath),
}

impl BezPathOrRect {

    pub fn to_path(&self) -> BezPath {
        match self {
            BezPathOrRect::Rect(rect) => {
                rect.to_kurbo().to_path(0.1)
            }
            BezPathOrRect::BezPath(path) => {path.clone()}
        }
    }

}

#[derive(Clone)]
struct ComputedBoxShadow {
    pub inset: bool,
    pub shape: BezPathOrRect,
    pub offset: Vec2,
    pub blur_radius: f64,
    pub color: Color,
}

#[derive(Clone)]
pub(crate) struct ComputedBoxShadows {
    outline: BezPathOrRect,
    box_shadows: Vec<ComputedBoxShadow>,
    border_box: Rectangle,
}

#[derive(Clone)]
pub(crate) struct CssComputedBorder {
    css_rect: CssRoundedRect,
    sides: [Option<BezPath>; 4],
    background: BezPath,
}

#[derive(Clone, PartialEq)]
struct BorderSpec {
    rect: Rectangle,
    width: TrblRectangle<f32>,
    radii: [(f32, f32); 4],
    scale_factor: f64,
    box_shadows: Vec<BoxShadow>,
}

#[derive(Clone, Default)]
pub(crate) enum ComputedBorder {
    CssComputedBorder(CssComputedBorder),
    Simple,
    #[default]
    None,
}

#[inline(always)]
fn from_taffy_point(p: taffy::Point<f32>) -> Point {
    Point {
        x: p.x as f64,
        y: p.y as f64,
    }
}
