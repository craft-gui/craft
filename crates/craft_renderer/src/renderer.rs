use crate::text_renderer_data::TextData;
use craft_primitives::geometry::Rectangle;
use craft_primitives::Color;
use craft_resource_manager::{ResourceIdentifier, ResourceManager};
use peniko::kurbo::Shape;
use peniko::{kurbo, BrushRef, Gradient};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::Arc;

#[derive(Clone)]
pub enum RenderCommand {
    DrawRect(Rectangle, Color),
    DrawRectOutline(Rectangle, Color, f64),
    DrawImage(Rectangle, ResourceIdentifier),
    DrawTinyVg(Rectangle, ResourceIdentifier, Option<Color>),
    DrawText(Weak<RefCell<dyn TextData>>, Rectangle, Option<TextScroll>, bool),
    PushLayer(Rectangle),
    PopLayer,
    FillBezPath(kurbo::BezPath, Brush),
    StartOverlay,
    EndOverlay,
}

#[derive(Clone, Debug)]
pub enum Brush {
    Color(Color),
    Gradient(Gradient),
}

impl<'a> From<&'a Brush> for BrushRef<'a> {
    fn from(brush: &'a Brush) -> Self {
        match brush {
            Brush::Color(color) => Self::Solid(*color),
            Brush::Gradient(gradient) => Self::Gradient(gradient),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextScroll {
    pub scroll_y: f32,
    pub scroll_height: f32,
}

impl TextScroll {
    pub fn new(scroll_y: f32, scroll_height: f32) -> Self {
        Self {
            scroll_y,
            scroll_height,
        }
    }
}

#[derive(Debug)]
enum SortedItem {
    Overlay(SortedCommands),
    Other(u32),
}

#[derive(Debug)]
pub struct SortedCommands {
    children: Vec<SortedItem>,
}

impl SortedCommands {
    pub fn draw(render_list: &RenderList, overlay_render: &SortedCommands, on_draw: &mut dyn FnMut(&RenderCommand)) {
        let mut others = Vec::new();
        let mut overlays = Vec::new();

        for child in &overlay_render.children {
            match child {
                SortedItem::Other(_) => others.push(child),
                SortedItem::Overlay(_) => overlays.push(child),
            }
        }

        for child in others {
            if let SortedItem::Other(command_index) = child {
                let command = render_list.commands.get(*command_index as usize).unwrap();
                on_draw(command);
            }
        }

        for child in overlays {
            if let SortedItem::Overlay(overlay) = child {
                Self::draw(render_list, overlay, on_draw);
            }
        }
    }
}

pub struct RenderList {
    pub targets: Vec<(u64, Rectangle)>,
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
    pub fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        if let Some(cull) = &self.cull {
            if !cull.intersects(&rectangle) {
                return;
            }
        }
        self.commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }


    pub fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        if let Some(cull) = &self.cull {
            if !cull.intersects(&bounding_box) {
                return;
            }
        }
        self.targets.push((id, bounding_box));
    }

    pub fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color, thickness: f64) {
        if let Some(cull) = &self.cull {
            if !cull.intersects(&rectangle) {
                return;
            }
        }
        self.commands.push(RenderCommand::DrawRectOutline(rectangle, outline_color, thickness));
    }

    pub fn fill_bez_path(&mut self, path: kurbo::BezPath, brush: Brush) {
        if let Some(cull) = &self.cull {
            if !cull.intersects(&Rectangle::from_kurbo(path.bounding_box())) {
                return;
            }
        }
        self.commands.push(RenderCommand::FillBezPath(path, brush));
    }

    pub fn draw_text(
        &mut self,
        component: Weak<RefCell<dyn TextData>>,
        rectangle: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        if let Some(cull) = &self.cull {
            if !cull.intersects(&rectangle) {
                return;
            }
        }
        self.commands.push(RenderCommand::DrawText(component, rectangle, text_scroll, show_cursor));
    }

    pub fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    pub fn draw_tiny_vg(
        &mut self,
        rectangle: Rectangle,
        resource_identifier: ResourceIdentifier,
        override_color: Option<Color>,
    ) {
        self.commands.push(RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color));
    }

    pub fn push_layer(&mut self, rect: Rectangle) {
        self.commands.push(RenderCommand::PushLayer(rect));
    }

    pub fn pop_layer(&mut self) {
        self.commands.push(RenderCommand::PopLayer);
    }

    pub fn start_overlay(&mut self) {
        self.commands.push(RenderCommand::StartOverlay);
    }

    pub fn end_overlay(&mut self) {
        self.commands.push(RenderCommand::EndOverlay);
    }

    pub fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.cull = cull;
    }
}

pub trait Renderer: Any {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);
    
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn sort_and_cull_render_list(&mut self, render_list: &mut RenderList) {
        fn should_cull(rectangle: &Rectangle, window_height: f32) -> bool {
            let cull_top = (rectangle.y + rectangle.height) < 0.0;
            let cull_bottom = rectangle.y > window_height;

            cull_top || cull_bottom
        }

        fn bounding_rect(render_command: &RenderCommand) -> Rectangle {
            match render_command {
                RenderCommand::DrawRect(rect, _)
                | RenderCommand::DrawRectOutline(rect, _, _)
                | RenderCommand::DrawImage(rect, _)
                | RenderCommand::DrawTinyVg(rect, _, _)
                | RenderCommand::DrawText(_, rect, _, _) => *rect,
                RenderCommand::FillBezPath(path, _) => Rectangle::from_kurbo(path.bounding_box()),
                _ => unreachable!("Cannot compute the bounding rect of this render command."),
            }
        }

        let window_height = self.surface_height();

        let mut current: *mut SortedCommands = &mut render_list.overlay;
        let mut stack: Vec<*mut SortedCommands> = vec![current];

        for (index, command) in render_list.commands.iter().enumerate() {
            match &command {
                RenderCommand::StartOverlay => {
                    // Overlay Start
                    unsafe {
                        (*current).children.push(SortedItem::Overlay(SortedCommands { children: vec![] }));
                        match (*current).children.last_mut() {
                            Some(SortedItem::Overlay(overlay)) => {
                                stack.push(overlay);
                            }
                            _ => {
                                panic!("OverlayRender stack corrupted");
                            }
                        }
                        current = *stack.last_mut().unwrap();
                    }
                }
                RenderCommand::EndOverlay => {
                    // Overlay End
                    stack.pop();
                    current = *stack.last_mut().unwrap();
                }

                // FIXME: If this is a clipping layer, and it is not in bounds we should discard all commands in the clip.
                RenderCommand::PushLayer(_) | RenderCommand::PopLayer => {
                    // Normal Draw Command
                    unsafe {
                        (*current).children.push(SortedItem::Other(index as u32));
                    }
                }

                _ => {
                    let bounding_rect = bounding_rect(command);
                    if !should_cull(&bounding_rect, window_height) {
                        unsafe {
                            (*current).children.push(SortedItem::Other(index as u32));
                        }
                    }
                }
            }
        }
    }
    fn prepare_render_list<'a>(
        &'a mut self,
        render_list: &'a mut RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
        //get_text_renderer: Box<dyn Fn(u64) -> Option<&'a TextRender> + 'a>,
    );

    fn submit(&mut self, resource_manager: Arc<ResourceManager>);
}
