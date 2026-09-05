pub use crate::screenshot::Screenshot;

use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use retgui_primitives::Color;
use retgui_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Shape};

use retgui_resource_manager::{ResourceId, ResourceManager};

use crate::render_command::{BoxShadowCmd, DrawBoxShadow, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
use crate::sort_commands::sort_render_list_internal;
use crate::text_renderer_data::{TextData, TextScroll};
use crate::{Brush, RenderCommand, TargetItem};

pub trait Renderer: Any {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);
    fn set_vsync(&mut self, _enabled: bool) {}

    fn render_list(&self) -> &RenderList;
    fn render_list_mut(&mut self) -> &mut RenderList;

    fn sort_render_list(&mut self) {
        let render_list = self.render_list_mut();
        TargetItem::sort_items_by_overlay_depth(&mut render_list.targets);
        sort_render_list_internal(render_list);
    }
    fn prepare(&mut self, resource_manager: Arc<ResourceManager>, window: Rectangle);

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
        self.render_list_mut().current_overlay_depth = 0;
        self.render_list_mut().transform = Affine::IDENTITY;
        self.render_list_mut().clip_stack.clear();
        self.render_list_mut().overlay_clip_stack.clear();
        self.render_list_mut().current_clip = self.render_list().cull;
    }

    #[inline(always)]
    fn set_transform(&mut self, transform: Affine) {
        self.render_list_mut().transform = transform;
    }

    #[inline(always)]
    fn get_transform(&self) -> Affine {
        self.render_list().transform
    }

    /// Returns the effective world-space clip while building the render list.
    #[inline(always)]
    fn get_clip(&self) -> Option<Rectangle> {
        self.render_list().current_clip
    }

    fn draw_circle(&mut self, circle: Circle, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawCircle(DrawCircleCmd {
                circle,
                brush,
                transform,
            }));
    }

    fn draw_circle_outline(&mut self, circle: Circle, outline_brush: Brush, thickness: f32) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawCircleOutline(DrawCircleOutlineCmd {
                circle,
                outline_brush,
                thickness,
                transform,
            }));
    }

    #[inline(always)]
    fn draw_rect(&mut self, rect: Rectangle, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawRect(DrawRectCmd {
                rect,
                brush,
                transform,
            }));
    }

    #[inline(always)]
    fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &bounding_box, self.render_list().cull.as_ref()) {
            return;
        }

        let mut bounding_box = Rectangle::from_kurbo(transform.transform_rect_bbox(bounding_box.to_kurbo()));
        if let Some(clip) = self.get_clip() {
            let Some(clipped) = bounding_box.intersection(&clip) else {
                return;
            };
            bounding_box = clipped;
        }

        let overlay_depth = self.render_list().current_overlay_depth;
        self.render_list_mut()
            .targets
            .push(TargetItem::new(id, bounding_box, overlay_depth));
    }

    #[inline(always)]
    fn draw_rect_outline(&mut self, rect: Rectangle, outline_brush: Brush, thickness: f64) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawRectOutline(DrawRectOutlineCmd {
                rect,
                outline_brush,
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

        self.render_list_mut()
            .commands
            .push(RenderCommand::FillBezPath(FillBezPathCmd {
                path,
                brush,
                transform,
            }));
    }

    #[inline(always)]
    fn stroke_bez_path(&mut self, path: BezPath, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_bez_path(&transform, &path, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::StrokeBezPath(StrokeBezPathCmd {
                path,
                brush,
                transform,
            }));
    }

    #[inline(always)]
    fn draw_text(
        &mut self,
        data: Rc<dyn TextData>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        self.draw_text_ref(&data, rect, text_scroll, show_cursor);
    }

    #[inline(always)]
    fn draw_text_ref(
        &mut self,
        data: &Rc<dyn TextData>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.render_list().cull.as_ref()) {
            return;
        }

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawText(DrawTextCmd {
                rect,
                data: data.clone(),
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

        self.render_list_mut()
            .commands
            .push(RenderCommand::DrawImage(DrawImageCmd {
                rect,
                resource_id,
                transform,
            }));
    }

    #[inline(always)]
    fn push_layer(&mut self, rect: Rectangle) {
        let transform = self.get_transform();
        let world_rect = Rectangle::from_kurbo(transform.transform_rect_bbox(rect.to_kurbo()));
        let previous_clip = self.render_list().current_clip;
        let next_clip = match previous_clip {
            Some(clip) => Some(
                clip.intersection(&world_rect)
                    .unwrap_or_else(|| Rectangle::new(0.0, 0.0, -1.0, -1.0)),
            ),
            None => Some(world_rect),
        };
        self.render_list_mut().clip_stack.push(previous_clip);
        self.render_list_mut().current_clip = next_clip;

        self.render_list_mut()
            .commands
            .push(RenderCommand::PushLayer(PushLayerCmd::Rect(rect, transform)));
    }

    fn push_layer_with_bez_path(&mut self, path: BezPath) {
        let transform = self.get_transform();
        let world_rect = Rectangle::from_kurbo(transform.transform_rect_bbox(path.bounding_box()));
        let previous_clip = self.render_list().current_clip;
        let next_clip = match previous_clip {
            Some(clip) => Some(
                clip.intersection(&world_rect)
                    .unwrap_or_else(|| Rectangle::new(0.0, 0.0, -1.0, -1.0)),
            ),
            None => Some(world_rect),
        };
        self.render_list_mut().clip_stack.push(previous_clip);
        self.render_list_mut().current_clip = next_clip;

        self.render_list_mut()
            .commands
            .push(RenderCommand::PushLayer(PushLayerCmd::BezPath(path, transform)));
    }

    #[inline(always)]
    fn pop_layer(&mut self) {
        self.render_list_mut().current_clip = self
            .render_list_mut()
            .clip_stack
            .pop()
            .expect("renderer layer stack underflow");
        self.render_list_mut().commands.push(RenderCommand::PopLayer);
    }

    fn start_overlay(&mut self) {
        let previous_clip = self.render_list().current_clip;
        self.render_list_mut().overlay_clip_stack.push(previous_clip);
        self.render_list_mut().current_clip = self.render_list().cull;
        self.render_list_mut().commands.push(RenderCommand::StartOverlay);
        self.render_list_mut().current_overlay_depth += 1;
    }

    fn end_overlay(&mut self) {
        self.render_list_mut().commands.push(RenderCommand::EndOverlay);
        self.render_list_mut().current_overlay_depth -= 1;
        self.render_list_mut().current_clip = self
            .render_list_mut()
            .overlay_clip_stack
            .pop()
            .expect("renderer overlay stack underflow");
    }

    #[inline(always)]
    fn draw_outset_box_shadow(&mut self, box_shadow: DrawBoxShadow) {
        let transform = self.get_transform();

        self.render_list_mut()
            .commands
            .push(RenderCommand::BoxShadowCmd(BoxShadowCmd {
                box_shadow,
                transform,
            }));
    }

    fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.render_list_mut().cull = cull;
        if self.render_list().clip_stack.is_empty() {
            self.render_list_mut().current_clip = cull;
        }
    }
}

#[inline(always)]
fn should_cull_rect(transform: &Affine, rect: &Rectangle, cull: Option<&Rectangle>) -> bool {
    if let Some(cull) = cull {
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
    if let Some(cull) = cull {
        let bb = path.bounding_box();
        let bb_transformed = transform.transform_rect_bbox(bb);

        if !bb_transformed.overlaps(cull.to_kurbo()) {
            return true;
        }
    }

    false
}
