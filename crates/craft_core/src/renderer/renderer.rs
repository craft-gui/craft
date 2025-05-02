use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::text::BufferGlyphs;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::{kurbo, BrushRef, Gradient};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawRect(Rectangle, Color),
    DrawRectOutline(Rectangle, Color),
    DrawImage(Rectangle, ResourceIdentifier),
    DrawTinyVg(Rectangle, ResourceIdentifier, Option<Color>),
    DrawText(BufferGlyphs, Rectangle, Option<TextScroll>, bool),
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
    Other(u32)
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
    pub commands: Vec<RenderCommand>,
    /// Stores a sorted list of render command handles. This gets set in `Renderer::sort_render_list`.
    pub overlay: SortedCommands,
}

impl RenderList {
    pub fn new() -> Self {
        Self { commands: Vec::new(), overlay: SortedCommands { children: vec![] } }
    }

    pub fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }
    pub fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        self.commands.push(RenderCommand::DrawRectOutline(rectangle, outline_color));
    }

    pub fn fill_bez_path(&mut self, path: kurbo::BezPath, brush: Brush) {
        self.commands.push(RenderCommand::FillBezPath(path, brush));
    }

    pub fn draw_text(
        &mut self,
        buffer_glyphs: BufferGlyphs,
        rectangle: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
    ) {
        self.commands.push(RenderCommand::DrawText(buffer_glyphs, rectangle, text_scroll, show_cursor));
    }
    pub fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    pub fn draw_tiny_vg(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier, override_color: Option<Color>) {
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
}

pub trait Renderer {
    // Surface Functions
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;
    #[allow(dead_code)]
    fn surface_height(&self) -> f32;
    fn resize_surface(&mut self, width: f32, height: f32);
    fn surface_set_clear_color(&mut self, color: Color);

    fn sort_render_list(&mut self, render_list: &mut RenderList) {
        let mut overlay_render = SortedCommands {
            children: vec![],
        };

        let mut current: *mut SortedCommands = &mut overlay_render;
        let mut stack: Vec<*mut SortedCommands> = vec![current];
        for (index, command) in render_list.commands.iter().enumerate() {
            match &command {
                RenderCommand::StartOverlay => {
                    // Overlay Start
                    unsafe {
                        (*current).children.push(SortedItem::Overlay(SortedCommands { children: vec![] }));
                        match (*current).children.last_mut(){
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
                _ => {
                    // Normal Draw Command
                    unsafe {
                        (*current).children.push(SortedItem::Other(index as u32));
                    }
                }
            }

        }
        render_list.overlay = overlay_render;
    }
    fn prepare_render_list(
        &mut self,
        render_list: RenderList,
        resource_manager: Arc<ResourceManager>,
        font_system: &mut FontSystem,
    );

    fn submit(&mut self, resource_manager: Arc<ResourceManager>);
}
