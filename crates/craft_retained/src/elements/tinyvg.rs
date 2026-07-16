//! Displays an TinyVg.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use craft_primitives::geometry::{Affine, BezPath, Point, Rectangle, Shape, TOLERANCE};
use craft_renderer::Brush;
use craft_renderer::renderer::Renderer;
use craft_resource_manager::{ResourceId, ResourceManager};
use peniko::color::AlphaColor;
use peniko::kurbo::{self, Stroke, StrokeOpts};
use peniko::{Color, Gradient};
use tinyvg_rs::TinyVg as TinyVgData;
use tinyvg_rs::color_table::{ColorTable, RgbaF32};
use tinyvg_rs::commands::{DrawCommand, Path, PathCommand, Point as TinyVgPoint, Style};
use tinyvg_rs::common::Unit;
use craft_resource_manager::resource_type::ResourceType;
use crate::app::{PENDING_RESOURCES, TAFFY_TREE};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::layout::TaffyTree;
use crate::layout::layout_context::{LayoutContext, TinyVgContext};
use crate::rgba;
use crate::text::text_context::TextContext;

/// Displays an TinyVg.
#[derive(Clone)]
pub struct TinyVg {
    pub inner: Rc<RefCell<TinyVgInner>>,
}

#[derive(Clone)]
pub struct TinyVgInner {
    is_tiny_vg_dirty: bool,
    resource_id: ResourceId,
    element_data: ElementData,
}

impl crate::elements::ElementData for TinyVgInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for TinyVg {}

impl Drop for TinyVgInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for TinyVg {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl ElementInternals for TinyVgInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_leaf_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(&mut self, renderer: &mut dyn Renderer, resource_manager: Arc<ResourceManager>, scale_factor: f64, _text_context: &mut TextContext) {
        if !self.is_visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);

        let computed_box_transformed = self.get_computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();

        let mut color = None;
        if self.style().get_color() != rgba(0, 0, 0, 0) {
            color = Some(self.style().get_color());
        }

        // Go through the tiny vg commands and draw them using the renderer trait.
        Self::draw_tiny_vg(
            renderer,
            content_rectangle.scale(scale_factor),
            &resource_manager,
            self.resource_id.clone(),
            &color,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TinyVg {
    pub fn new(resource_id: ResourceId) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<TinyVgInner>>| {
            RefCell::new(TinyVgInner {
                is_tiny_vg_dirty: false,
                resource_id: resource_id.clone(),
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::TinyVg(TinyVgContext::new(resource_id.clone())));
        inner.borrow_mut().element_data.create_layout_node(layout_context);
        inner.borrow_mut().style_mut().set_color(Color::TRANSPARENT);

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_id, ResourceType::TinyVg));
        });

        Self { inner }
    }

    pub fn dummy() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<TinyVgInner>>| {
            RefCell::new(TinyVgInner {
                is_tiny_vg_dirty: false,
                resource_id: ResourceId::DUMMY,
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::TinyVg(TinyVgContext::new(ResourceId::DUMMY)));
        inner.borrow_mut().element_data.create_layout_node(layout_context);
        inner.borrow_mut().style_mut().set_color(Color::TRANSPARENT);

        Self { inner }
    }

    pub fn resource_id(self, resource_id: ResourceId) -> Self {
        self.inner.borrow_mut().set_resource_id(resource_id);
        self
    }

    pub fn get_resource_id(&self) -> ResourceId {
        self.inner.borrow().get_resource_id().clone()
    }
}

impl TinyVgInner {
    pub fn set_resource_id(&mut self, resource_id: ResourceId) {
        self.is_tiny_vg_dirty = true;
        self.resource_id = resource_id.clone();

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_id.clone(), ResourceType::TinyVg));
        });

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let context = LayoutContext::TinyVg(TinyVgContext::new(resource_id));
            let node = self
                .element_data
                .layout
                .taffy_node_id
                .expect("Failed to get TinyVg node");
            taffy_tree.set_node_context(node, Some(context));
        });
    }

    pub fn get_resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub(crate) fn draw_tiny_vg(
        renderer: &mut dyn Renderer,
        rectangle: Rectangle,
        resource_manager: &Arc<ResourceManager>,
        resource_id: ResourceId,
        override_color: &Option<Color>,
    ) {
        let resource = resource_manager.get(&resource_id);
        if resource.is_none() {
            return;
        }
        let resource = &resource.unwrap();
        if resource.resource_type != ResourceType::TinyVg {
            return;
        }

        let tiny_vg = resource.data.downcast_ref::<TinyVgData>().unwrap();

        let vg_transform = Affine::IDENTITY;
        let mut svg_width = tiny_vg.header.width as f32;
        let mut svg_height = tiny_vg.header.height as f32;

        if tiny_vg.header.width == 0 {
            svg_width = rectangle.width;
        }
        if tiny_vg.header.height == 0 {
            svg_height = rectangle.height;
        }

        let vg_transform = vg_transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
        let vg_transform = vg_transform.pre_scale_non_uniform(
            rectangle.width as f64 / svg_width as f64,
            rectangle.height as f64 / svg_height as f64,
        );

        let old_transform = renderer.get_transform();
        renderer.set_transform(vg_transform * old_transform);

        for command in &tiny_vg.draw_commands {
            match command {
                DrawCommand::FillPolygon(data) => {
                    let mut path = BezPath::new();
                    if let Some(first) = data.points.first() {
                        path.move_to(to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(to_kurbo_point(*point));
                        }
                        path.close_path();
                    }
                    let brush = get_brush(&data.style, &tiny_vg.color_table, override_color);
                    fill_path(renderer, path, brush);
                }
                DrawCommand::FillRectangles(data) => {
                    let brush = get_brush(&data.style, &tiny_vg.color_table, override_color);
                    for rectangle in &data.rectangles {
                        let rect =
                            kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.width.0, rectangle.height.0);
                        fill_path(renderer, rect.into_path(TOLERANCE), brush.clone());
                    }
                }
                DrawCommand::FillPath(data) => {
                    draw_path(
                        renderer,
                        &data.path,
                        &data.style,
                        None,
                        &tiny_vg.color_table,
                        override_color
                    );
                }
                DrawCommand::DrawLines(data) => {
                    let brush = get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    for line in &data.lines {
                        path.move_to(to_kurbo_point(line.start));
                        path.line_to(to_kurbo_point(line.end));
                    }

                    stroke_path(renderer, &path, data.line_width.0, brush);
                }
                DrawCommand::DrawLineLoop(data) => {
                    let brush = get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    if let Some(first) = data.points.first() {
                        path.move_to(to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(to_kurbo_point(*point));
                        }
                        path.close_path();
                    }

                    stroke_path(renderer, &path, data.line_width.0, brush);
                }
                DrawCommand::DrawLineStrip(data) => {
                    let brush = get_brush(&data.style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    if let Some(first) = data.points.first() {
                        path.move_to(to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(to_kurbo_point(*point));
                        }
                    }

                    stroke_path(renderer, &path, data.line_width.0, brush);
                }
                DrawCommand::DrawLinePath(data) => {
                    draw_path(
                        renderer,
                        &data.path,
                        &data.style,
                        Some(&data.line_width),
                        &tiny_vg.color_table,
                        override_color
                    );
                }
                DrawCommand::OutlineFillPolygon(data) => {
                    let mut path = BezPath::new();
                    if let Some(first) = data.points.first() {
                        path.move_to(to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(to_kurbo_point(*point));
                        }
                        path.close_path();
                    }

                    let fill_brush = get_brush(&data.fill_style, &tiny_vg.color_table, override_color);
                    fill_path(renderer, path.clone(), fill_brush);

                    let line_brush = get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    stroke_path(renderer, &path, data.line_width.0, line_brush);
                }
                DrawCommand::OutlineFillRectangles(data) => {
                    let fill_brush = get_brush(&data.fill_style, &tiny_vg.color_table, override_color);
                    let line_brush = get_brush(&data.line_style, &tiny_vg.color_table, override_color);

                    for rectangle in &data.rectangles {
                        let rect =
                            kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.width.0, rectangle.height.0);
                        let rect_path = rect.into_path(TOLERANCE);
                        fill_path(renderer, rect_path.clone(), fill_brush.clone());
                        stroke_path(renderer, &rect_path, data.line_width.0, line_brush.clone());
                    }
                }
                DrawCommand::OutlineFillPath(data) => {
                    draw_path(
                        renderer,
                        &data.path,
                        &data.fill_style,
                        None,
                        &tiny_vg.color_table,
                        override_color
                    );
                    draw_path(
                        renderer,
                        &data.path,
                        &data.line_style,
                        Some(&data.line_width),
                        &tiny_vg.color_table,
                        override_color
                    );
                }
                DrawCommand::TextHint(_data) => {}
            }
        }

        renderer.set_transform(old_transform);
    }
}

fn to_kurbo_point(point: TinyVgPoint) -> Point {
    Point::new(point.x.0, point.y.0)
}

fn to_peniko_color(color: RgbaF32) -> Color {
    Color::from(AlphaColor::new([color.0, color.1, color.2, color.3]))
}

fn fill_path(renderer: &mut dyn Renderer, path: BezPath, brush: Brush) {
    renderer.fill_bez_path(path, brush);
}

fn stroke_path(renderer: &mut dyn Renderer, path: &BezPath, line_width: f64, brush: Brush) {
    let outline = kurbo::stroke(path, &Stroke::new(line_width), &StrokeOpts::default(), TOLERANCE);
    renderer.fill_bez_path(outline, brush);
}

fn draw_path(
    renderer: &mut dyn Renderer,
    path: &Path,
    fill_style: &Style,
    line_width: Option<&Unit>,
    color_table: &ColorTable,
    override_color: &Option<Color>
) {
    let bezier_path = assemble_path(path);
    let brush = get_brush(fill_style, color_table, override_color);

    if let Some(line_width) = line_width {
        stroke_path(renderer, &bezier_path, line_width.0, brush);
    } else {
        fill_path(renderer, bezier_path, brush);
    }
}

fn assemble_path(path: &Path) -> BezPath {
    let mut bezier_path = BezPath::new();

    for segment in &path.segments {
        let mut current = segment.start;
        bezier_path.move_to(to_kurbo_point(current));

        for path_command in &segment.path_commands {
            match path_command {
                PathCommand::Line(point, _line_width) => {
                    bezier_path.line_to(to_kurbo_point(*point));
                    current = current.move_to(point);
                }
                PathCommand::HorizontalLine(horizontal, _line_width) => {
                    let horizontal_end_point = TinyVgPoint {
                        x: *horizontal,
                        y: current.y,
                    };
                    bezier_path.line_to(to_kurbo_point(horizontal_end_point));
                    current = current.move_to(&horizontal_end_point);
                }
                PathCommand::VerticalLine(vertical, _line_width) => {
                    let vertical_end_point = TinyVgPoint {
                        x: current.x,
                        y: *vertical,
                    };
                    bezier_path.line_to(to_kurbo_point(vertical_end_point));
                    current = current.move_to(&vertical_end_point);
                }
                PathCommand::CubicBezier(cubic_bezier, _line_width) => {
                    let end = cubic_bezier.point_1;
                    bezier_path.curve_to(
                        (cubic_bezier.control_point_0.x.0, cubic_bezier.control_point_0.y.0),
                        (cubic_bezier.control_point_1.x.0, cubic_bezier.control_point_1.y.0),
                        (end.x.0, end.y.0),
                    );
                    current = current.move_to(&end);
                }
                PathCommand::ArcCircle(arc_circle, _line_width) => {
                    let arc_start = to_kurbo_point(current);
                    let arc_end = to_kurbo_point(arc_circle.target);

                    let arc = kurbo::SvgArc {
                        from: arc_start,
                        to: arc_end,
                        radii: kurbo::Vec2::new(arc_circle.radius.0, arc_circle.radius.0),
                        x_rotation: 0.0,
                        large_arc: arc_circle.large_arc,
                        sweep: arc_circle.sweep,
                    };

                    if let Some(arc) = kurbo::Arc::from_svg_arc(&arc) {
                        bezier_path.extend(arc.append_iter(TOLERANCE));
                    }

                    current = current.move_to(&arc_circle.target);
                }
                PathCommand::ArcEllipse(arc_ellipse, _line_width) => {
                    let arc_start = to_kurbo_point(current);
                    let arc_end = to_kurbo_point(arc_ellipse.target);

                    let arc = kurbo::SvgArc {
                        from: arc_start,
                        to: arc_end,
                        radii: kurbo::Vec2::new(arc_ellipse.radius_x.0, arc_ellipse.radius_y.0),
                        x_rotation: 0.0,
                        large_arc: arc_ellipse.large_arc,
                        sweep: arc_ellipse.sweep,
                    };

                    if let Some(arc) = kurbo::Arc::from_svg_arc(&arc) {
                        bezier_path.extend(arc.append_iter(TOLERANCE));
                    }

                    current = current.move_to(&arc_ellipse.target);
                }
                PathCommand::ClosePath => {
                    bezier_path.close_path();
                }
                PathCommand::QuadraticBezier(quadratic_bezier, _line_width) => {
                    let end = quadratic_bezier.point_1;
                    bezier_path.quad_to(
                        (
                            to_kurbo_point(quadratic_bezier.control_point).x,
                            to_kurbo_point(quadratic_bezier.control_point).y,
                        ),
                        (to_kurbo_point(end).x, to_kurbo_point(end).y),
                    );

                    current = current.move_to(&end);
                }
            }
        }
    }

    bezier_path
}

fn get_brush(fill_style: &Style, color_table: &ColorTable, override_color: &Option<Color>) -> Brush {
    if let Some(override_color) = override_color {
        return Brush::Color(*override_color);
    }

    match fill_style {
        Style::FlatColor(flat_colored) => {
            let color = color_table[flat_colored.color_index as usize];
            Brush::Color(to_peniko_color(color))
        }
        Style::LinearGradient(linear_gradient) => {
            let color_0 = color_table[linear_gradient.color_index_0 as usize];
            let color_1 = color_table[linear_gradient.color_index_1 as usize];

            let start = to_kurbo_point(linear_gradient.point_0);
            let end = to_kurbo_point(linear_gradient.point_1);

            let linear =
                Gradient::new_linear(start, end).with_stops([to_peniko_color(color_0), to_peniko_color(color_1)]);
            Brush::Gradient(linear)
        }
        Style::RadialGradient(radial_gradient) => {
            let color_0 = color_table[radial_gradient.color_index_0 as usize];
            let color_1 = color_table[radial_gradient.color_index_1 as usize];

            let center = to_kurbo_point(radial_gradient.point_0);
            let edge = to_kurbo_point(radial_gradient.point_1);
            let radius = center.distance(edge);

            let radial = Gradient::new_radial(center, radius as f32)
                .with_stops([to_peniko_color(color_0), to_peniko_color(color_1)]);
            Brush::Gradient(radial)
        }
    }
}
