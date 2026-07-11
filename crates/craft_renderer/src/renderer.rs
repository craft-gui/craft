use std::any::Any;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::Arc;

use craft_primitives::Color;
use craft_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Shape};

use craft_resource_manager::{ResourceId, ResourceManager};
use crate::render_command::{BoxShadowCmd, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, DrawTinyVgCmd, FillBezPathCmd, PushLayerCmd, SetTransformCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
use crate::{Brush, RenderCommand, TargetItem};
pub use crate::screenshot::Screenshot;
use crate::sort_commands::sort_and_cull_render_list_internal;
use crate::text_renderer_data::{TextData, TextScroll};

pub trait Renderer: Any {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn render_list(&self) -> &RenderList;
    fn render_list_mut(&mut self) -> &mut RenderList;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn sort_and_cull_render_list(&mut self) {
        let surface_height = self.surface_height();
        sort_and_cull_render_list_internal(surface_height, self.render_list_mut());
    }
    fn prepare<'a>(
        &mut self,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    );

    fn submit(&mut self, resource_manager: Arc<ResourceManager>);

    fn screenshot(&self) -> Screenshot {
        Screenshot {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.render_list_mut().targets.clear();
        self.render_list_mut().commands.clear();
        self.render_list_mut().overlay.children.clear();
    }

    #[inline(always)]
    fn set_transform(&mut self, transform: Affine) {
        self.render_list_mut().commands
            .push(RenderCommand::SetTransform(SetTransformCmd { transform }));
    }

    fn draw_circle(&mut self, circle: Circle, color: Color) {
        if let Some(cull) = &self.render_list().cull
            && !circle.intersects_rect(cull)
        {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::DrawCircle(DrawCircleCmd { circle, color }));
    }

    fn draw_circle_outline(&mut self, circle: Circle, outline_color: Color, thickness: f32) {
        if let Some(cull) = &self.render_list().cull
            && !circle.intersects_rect(cull)
        {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::DrawCircleOutline(DrawCircleOutlineCmd {
                circle,
                outline_color,
                thickness,
            }));
    }

    #[inline(always)]
    fn draw_rect(&mut self, rect: Rectangle, color: Color) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.render_list_mut().commands.push(RenderCommand::DrawRect(DrawRectCmd { rect, color }));
    }

    #[inline(always)]
    fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        if let Some(cull) = self.render_list().cull.clone()
            && !cull.intersects(&bounding_box)
        {
            return;
        }
        let overlay_depth = self.render_list().current_overlay_depth;
        self.render_list_mut().targets
            .push(TargetItem::new(id, bounding_box, overlay_depth));
    }

    #[inline(always)]
    fn draw_rect_outline(&mut self, rect: Rectangle, outline_color: Color, thickness: f64) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.render_list_mut().commands.push(RenderCommand::DrawRectOutline(DrawRectOutlineCmd {
            rect,
            outline_color,
            thickness,
        }));
    }

    #[inline(always)]
    fn fill_bez_path(&mut self, path: BezPath, brush: Brush) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&Rectangle::from_kurbo(path.bounding_box()))
        {
            return;
        }
        self.render_list_mut().commands
            .push(RenderCommand::FillBezPath(FillBezPathCmd { path, brush }));
    }

    #[inline(always)]
    fn stroke_bez_path(&mut self, path: BezPath, brush: Brush) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&Rectangle::from_kurbo(path.bounding_box()))
        {
            return;
        }
        self.render_list_mut().commands
            .push(RenderCommand::StrokeBezPath(StrokeBezPathCmd { path, brush }));
    }

    #[inline(always)]
    fn draw_text(
        &mut self,
        data: Weak<RefCell<dyn TextData>>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.render_list_mut().commands.push(RenderCommand::DrawText(DrawTextCmd {
            rect,
            data,
            text_scroll,
            show_cursor,
        }));
    }

    #[inline(always)]
    fn draw_image(&mut self, rect: Rectangle, resource_id: ResourceId) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.render_list_mut().commands
            .push(RenderCommand::DrawImage(DrawImageCmd { rect, resource_id }));
    }

    #[inline(always)]
    fn draw_tiny_vg(&mut self, rect: Rectangle, resource_id: ResourceId, override_color: Option<Color>) {
        if let Some(cull) = &self.render_list().cull
            && !cull.intersects(&rect)
        {
            return;
        }
        self.render_list_mut().commands.push(RenderCommand::DrawTinyVg(DrawTinyVgCmd {
            rect,
            resource_id,
            override_color,
        }));
    }

    #[inline(always)]
    fn push_layer(&mut self, rect: Rectangle) {
        self.render_list_mut().commands.push(RenderCommand::PushLayer(PushLayerCmd::Rect(rect)));
    }

    fn push_layer_with_bez_path(&mut self, path: BezPath) {
        self.render_list_mut().commands
            .push(RenderCommand::PushLayer(PushLayerCmd::BezPath(path)));
    }

    #[inline(always)]
    fn pop_layer(&mut self) {
        self.render_list_mut().commands.push(RenderCommand::PopLayer);
    }

    fn start_overlay(&mut self) {
        self.render_list_mut().commands.push(RenderCommand::StartOverlay);
        self.render_list_mut().current_overlay_depth += 1;
    }

    fn end_overlay(&mut self) {
        self.render_list_mut().commands.push(RenderCommand::EndOverlay);
        self.render_list_mut().current_overlay_depth -= 1;
    }

    #[inline(always)]
    fn draw_outset_box_shadow(
        &mut self,
        box_shadow: BoxShadowCmd,
    ) {
        self.render_list_mut().commands.push(RenderCommand::BoxShadowCmd(box_shadow));
    }

    fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.render_list_mut().cull = cull;
    }
}
