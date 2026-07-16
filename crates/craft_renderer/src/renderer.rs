use std::any::Any;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::Arc;

use craft_primitives::Color;
use craft_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Shape};

use craft_resource_manager::{ResourceId, ResourceManager};
use crate::render_command::{BoxShadowCmd, DrawBoxShadow, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
use crate::{Brush, RenderCommand, TargetItem};
pub use crate::screenshot::Screenshot;
use crate::sort_commands::sort_render_list_internal;
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

    fn sort_render_list(&mut self) {
        sort_render_list_internal(self.render_list_mut());
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
        self.render_list_mut().transform = Affine::IDENTITY;
    }

    #[inline(always)]
    fn set_transform(&mut self, transform: Affine) {
        self.render_list_mut().transform = transform;
    }

    #[inline(always)]
    fn get_transform(&self) -> Affine {
        self.render_list().transform
    }

    fn draw_circle(&mut self, circle: Circle, color: Color) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::DrawCircle(DrawCircleCmd { circle, color, transform }));
    }

    fn draw_circle_outline(&mut self, circle: Circle, outline_color: Color, thickness: f32) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::DrawCircleOutline(DrawCircleOutlineCmd {
                circle,
                outline_color,
                thickness,
                transform,
            }));
    }

    #[inline(always)]
    fn draw_rect(&mut self, rect: Rectangle, color: Color) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands.push(RenderCommand::DrawRect(DrawRectCmd { rect, color, transform }));
    }

    #[inline(always)]
    fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &bounding_box, self.render_list().cull.as_ref()) {
            return;
        }

        let overlay_depth = self.render_list().current_overlay_depth;
        self.render_list_mut().targets
            .push(TargetItem::new(id, bounding_box, overlay_depth));
    }

    #[inline(always)]
    fn draw_rect_outline(&mut self, rect: Rectangle, outline_color: Color, thickness: f64) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands.push(RenderCommand::DrawRectOutline(DrawRectOutlineCmd {
            rect,
            outline_color,
            thickness,
            transform,
        }));
    }

    #[inline(always)]
    fn fill_bez_path(&mut self, path: BezPath, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_bez_path(&transform, &path, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::FillBezPath(FillBezPathCmd { path, brush, transform }));
    }

    #[inline(always)]
    fn stroke_bez_path(&mut self, path: BezPath, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_bez_path(&transform, &path, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::StrokeBezPath(StrokeBezPathCmd { path, brush, transform }));
    }

    #[inline(always)]
    fn draw_text(
        &mut self,
        data: Weak<RefCell<dyn TextData>>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands.push(RenderCommand::DrawText(DrawTextCmd {
            rect,
            data,
            text_scroll,
            show_cursor,
            transform,
        }));
    }

    #[inline(always)]
    fn draw_image(&mut self, rect: Rectangle, resource_id: ResourceId) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut().commands
            .push(RenderCommand::DrawImage(DrawImageCmd { rect, resource_id, transform: Default::default() }));
    }

    #[inline(always)]
    fn push_layer(&mut self, rect: Rectangle) {
        let transform = self.get_transform();

        self.render_list_mut().commands.push(RenderCommand::PushLayer(PushLayerCmd::Rect(rect, transform)));
    }

    fn push_layer_with_bez_path(&mut self, path: BezPath) {
        let transform = self.get_transform();

        self.render_list_mut().commands
            .push(RenderCommand::PushLayer(PushLayerCmd::BezPath(path, transform)));
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
        box_shadow: DrawBoxShadow,
    ) {
        let transform = self.get_transform();

        self.render_list_mut().commands.push(RenderCommand::BoxShadowCmd(BoxShadowCmd {
            box_shadow,
            transform,
        }));
    }

    fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.render_list_mut().cull = cull;
    }
}

#[inline(always)]
fn should_cull_rect(transform: &Affine, rect: &Rectangle, cull: Option<&Rectangle>) -> bool {
    if let Some(cull) = cull
    {
        let bb = rect.to_kurbo();
        let bb_transformed = transform.transform_rect_bbox(bb);

        if !bb_transformed.overlaps(cull.to_kurbo()) {
            return true;
        }
    }

    false
}

#[inline(always)]
fn should_cull_bez_path(transform: &Affine, path: &BezPath, cull: Option<&Rectangle>) -> bool {
    if let Some(cull) = cull
    {
        let bb = path.bounding_box();
        let bb_transformed = transform.transform_rect_bbox(bb);

        if !bb_transformed.overlaps(cull.to_kurbo()) {
            return true;
        }
    }

    false
}

