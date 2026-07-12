use craft_primitives::geometry::{Affine, Rectangle};

use crate::sort_commands::SortedCommands;
use crate::{RenderCommand, TargetItem};

pub struct RenderList {
    pub current_overlay_depth: u64,
    pub targets: Vec<TargetItem>,
    pub commands: Vec<RenderCommand>,
    /// Stores a sorted list of render command handles. This gets set in `Renderer::sort_render_list`.
    pub overlay: SortedCommands,
    pub cull: Option<Rectangle>,
    pub transform: Affine
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
            transform: Affine::IDENTITY,
        }
    }
}
