use std::cell::RefCell;
use std::rc::Weak;

use peniko::Color;

use craft_primitives::geometry::{BezPath, Rectangle, Vec2};

use craft_resource_manager::ResourceIdentifier;

use crate::Brush;
use crate::text_renderer_data::{TextData, TextScroll};

#[derive(Clone)]
pub enum RenderCommand {
    DrawRect(DrawRectCmd),
    DrawRectOutline(DrawRectOutlineCmd),
    DrawImage(DrawImageCmd),
    DrawTinyVg(DrawTinyVgCmd),
    DrawText(DrawTextCmd),
    PushLayer(PushLayerCmd),
    PopLayer,
    FillBezPath(FillBezPathCmd),
    StartOverlay,
    EndOverlay,
    BoxShadowCmd(BoxShadowCmd),
}

#[derive(Clone)]
pub struct DrawRectCmd {
    pub rect: Rectangle,
    pub color: Color
}

#[derive(Clone)]
pub struct DrawRectOutlineCmd {
    pub rect: Rectangle,
    pub outline_color: Color,
    pub thickness: f64
}

#[derive(Clone)]
pub struct DrawImageCmd {
    pub rect: Rectangle,
    pub resource_id: ResourceIdentifier
}

#[derive(Clone)]
pub struct DrawTinyVgCmd {
    pub rect: Rectangle,
    pub resource_id: ResourceIdentifier,
    pub override_color: Option<Color>
}

#[derive(Clone)]
pub struct DrawTextCmd {
    pub rect: Rectangle,
    pub data: Weak<RefCell<dyn TextData>>,
    pub text_scroll: Option<TextScroll>,
    pub show_cursor: bool
}

#[derive(Clone)]
pub struct PushLayerCmd {
    pub rect: Rectangle
}

#[derive(Clone)]
pub struct FillBezPathCmd {
    pub path: BezPath,
    pub brush: Brush
}

#[derive(Clone)]
pub struct BoxShadowCmd {
    pub inset: bool,
    pub offset: Vec2,
    pub outline: BezPath,
    pub path: BezPath,
    pub blur_radius: f64,
    pub color: Color,
    pub border_box: Rectangle,
}