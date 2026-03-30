use std::cell::RefCell;
use std::rc::Weak;

use peniko::Color;

use craft_primitives::geometry::{Affine, BezPath, Rectangle, Shape};
use craft_resource_manager::ResourceIdentifier;

use crate::render_command::{BoxShadowCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, DrawTinyVgCmd, FillBezPathCmd, PushLayerCmd};
use crate::sort_commands::SortedCommands;
use crate::text_renderer_data::{TextData, TextScroll};
use crate::{Brush, RenderCommand, TargetItem};

pub struct RenderList {
    current_overlay_depth: u64,
    pub targets: Vec<TargetItem>,
    pub commands: Vec<RenderCommand>,
    /// Stores a sorted list of render command handles. This gets set in `Renderer::sort_render_list`.
    pub overlay: SortedCommands,
    cull: Option<Rectangle>,
}

impl Default for RenderList {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderList {
    pub fn new() -> Self {
        Self {
            current_overlay_depth: 0,
            targets: Vec::new(),
            commands: Vec::new(),
            overlay: SortedCommands { children: vec![] },
            cull: None,
        }
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.commands.clear();
        self.overlay.children.clear();
    }

    #[inline(always)]
    pub fn draw_rect(&mut self, rect: Rectangle, color: Color) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.commands.push(RenderCommand::DrawRect(DrawRectCmd { rect, color }));
    }

    #[inline(always)]
    pub fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&bounding_box)
        {
            return;
        }
        self.targets
            .push(TargetItem::new(id, bounding_box, self.current_overlay_depth));
    }

    #[inline(always)]
    pub fn draw_rect_outline(&mut self, rect: Rectangle, outline_color: Color, thickness: f64) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.commands.push(RenderCommand::DrawRectOutline(DrawRectOutlineCmd {
            rect,
            outline_color,
            thickness,
        }));
    }

    #[inline(always)]
    pub fn fill_bez_path(&mut self, path: BezPath, brush: Brush) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&Rectangle::from_kurbo(path.bounding_box()))
        {
            return;
        }
        self.commands
            .push(RenderCommand::FillBezPath(FillBezPathCmd { path, brush }));
    }

    #[inline(always)]
    pub fn draw_text(
        &mut self,
        data: Weak<RefCell<dyn TextData>>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.commands.push(RenderCommand::DrawText(DrawTextCmd {
            rect,
            data,
            text_scroll,
            show_cursor,
        }));
    }

    #[inline(always)]
    pub fn draw_image(&mut self, rect: Rectangle, resource_id: ResourceIdentifier) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.commands
            .push(RenderCommand::DrawImage(DrawImageCmd { rect, resource_id }));
    }

    #[inline(always)]
    pub fn draw_tiny_vg(&mut self, rect: Rectangle, resource_id: ResourceIdentifier, override_color: Option<Color>) {
        if let Some(cull) = &self.cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.commands.push(RenderCommand::DrawTinyVg(DrawTinyVgCmd {
            rect,
            resource_id,
            override_color,
        }));
    }

    #[inline(always)]
    pub fn push_layer(&mut self, rect: Rectangle) {
        self.commands.push(RenderCommand::PushLayer(PushLayerCmd::Rect(rect)));
    }

    pub fn push_layer_with_bez_path(&mut self, path: BezPath) {
        self.commands
            .push(RenderCommand::PushLayer(PushLayerCmd::BezPath(path)));
    }

    #[inline(always)]
    pub fn pop_layer(&mut self) {
        self.commands.push(RenderCommand::PopLayer);
    }

    pub fn start_overlay(&mut self) {
        self.commands.push(RenderCommand::StartOverlay);
        self.current_overlay_depth += 1;
    }

    pub fn end_overlay(&mut self) {
        self.commands.push(RenderCommand::EndOverlay);
        self.current_overlay_depth -= 1;
    }

    #[inline(always)]
    pub fn draw_outset_box_shadow(
        &mut self,
        box_shadow: BoxShadowCmd,
        //rectangle: Rectangle,
    ) {
        /*        if let Some(cull) = &self.cull
            && !cull.intersects(&rectangle)
        {
            return;
        }*/
        self.commands.push(RenderCommand::BoxShadowCmd(box_shadow));
    }

    pub fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.cull = cull;
    }
}
