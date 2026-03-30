use crate::{RenderCommand, RenderList};
use craft_primitives::geometry::{Rectangle, Shape};

#[derive(Debug)]
pub enum SortedItem {
    Overlay(SortedCommands),
    Other(u32),
}

#[derive(Debug)]
pub struct SortedCommands {
    pub children: Vec<SortedItem>,
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

pub(crate) fn sort_and_cull_render_list_internal(surface_height: f32, render_list: &mut RenderList) {
    fn should_cull(rectangle: &Rectangle, window_height: f32) -> bool {
        let cull_top = (rectangle.y + rectangle.height) < 0.0;
        let cull_bottom = rectangle.y > window_height;

        cull_top || cull_bottom
    }

    fn bounding_rect(render_command: &RenderCommand) -> Rectangle {
        match render_command {
            RenderCommand::DrawRect(cmd) => cmd.rect,
            RenderCommand::DrawRectOutline(cmd) => cmd.rect,
            RenderCommand::DrawImage(cmd) => cmd.rect,
            RenderCommand::DrawTinyVg(cmd) => cmd.rect,
            RenderCommand::DrawText(cmd) => cmd.rect,
            RenderCommand::FillBezPath(cmd) => Rectangle::from_kurbo(cmd.path.bounding_box()),
            RenderCommand::BoxShadowCmd(cmd) => {
                let bounding_box = cmd.path.bounding_box();
                Rectangle::new(
                    (bounding_box.x0 + cmd.offset.x) as f32,
                    (bounding_box.y0 + cmd.offset.y) as f32,
                    (bounding_box.x1 + cmd.offset.x) as f32,
                    (bounding_box.y1 + cmd.offset.y) as f32,
                )
            }
            _ => unreachable!("Cannot compute the bounding rect of this render command."),
        }
    }

    let window_height = surface_height;

    let mut current: *mut SortedCommands = &mut render_list.overlay;
    let mut stack: Vec<*mut SortedCommands> = vec![current];

    for (index, command) in render_list.commands.iter().enumerate() {
        match &command {
            RenderCommand::StartOverlay => {
                // Overlay Start
                unsafe {
                    (*current)
                        .children
                        .push(SortedItem::Overlay(SortedCommands { children: vec![] }));
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
