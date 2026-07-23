use std::cell::RefCell;
use std::rc::Weak;

use peniko::Color;
use craft_primitives::brush::Brush;
use craft_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Vec2};

use craft_resource_manager::ResourceId;

use crate::text_renderer_data::{TextData, TextScroll};

#[derive(Clone)]
pub enum RenderCommand {
    DrawCircle(DrawCircleCmd),
    DrawCircleOutline(DrawCircleOutlineCmd),
    DrawRect(DrawRectCmd),
    DrawRectOutline(DrawRectOutlineCmd),
    DrawImage(DrawImageCmd),
    DrawText(DrawTextCmd),
    PushLayer(PushLayerCmd),
    PopLayer,
    FillBezPath(FillBezPathCmd),
    StartOverlay,
    StrokeBezPath(StrokeBezPathCmd),
    EndOverlay,
    BoxShadowCmd(BoxShadowCmd),
}

#[derive(Clone)]
pub struct DrawCircleCmd {
    pub circle: Circle,
    pub brush: Brush,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawCircleOutlineCmd {
    pub circle: Circle,
    pub outline_brush: Brush,
    pub thickness: f32,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawRectCmd {
    pub rect: Rectangle,
    pub brush: Brush,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawRectOutlineCmd {
    pub rect: Rectangle,
    pub outline_brush: Brush,
    pub thickness: f64,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawImageCmd {
    pub rect: Rectangle,
    pub resource_id: ResourceId,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawTextCmd {
    pub rect: Rectangle,
    pub data: Weak<RefCell<dyn TextData>>,
    pub text_scroll: Option<TextScroll>,
    pub show_cursor: bool,
    pub transform: Affine,
}

#[derive(Clone)]
pub enum PushLayerCmd {
    BezPath(BezPath, Affine),
    Rect(Rectangle, Affine),
}

#[derive(Clone)]
pub struct FillBezPathCmd {
    pub path: BezPath,
    pub brush: Brush,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct StrokeBezPathCmd {
    pub path: BezPath,
    pub brush: Brush,
    pub transform: Affine,
}

#[derive(Clone)]
pub struct DrawBoxShadow {
    pub inset: bool,
    pub offset: Vec2,
    pub outline: BezPath,
    pub path: BezPath,
    pub blur_radius: f64,
    pub color: Color,
    pub border_box: Rectangle,
}

#[derive(Clone)]
pub struct BoxShadowCmd {
    pub box_shadow: DrawBoxShadow,
    pub transform: Affine,
}
