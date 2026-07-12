use crate::{RenderCommand, RenderList};

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

pub(crate) fn sort_render_list_internal(render_list: &mut RenderList) {
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

            RenderCommand::PushLayer(_) | RenderCommand::PopLayer => {
                // Normal Draw Command
                unsafe {
                    (*current).children.push(SortedItem::Other(index as u32));
                }
            }

            _ => {
                unsafe {
                    (*current).children.push(SortedItem::Other(index as u32));
                }
            }
        }
    }
}
