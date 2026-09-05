use std::collections::HashMap;
use std::collections::hash_map::Values;

use retgui_resource_manager::{ResourceId as RetGuiResourceId, ResourceId};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RendererResourceId(pub u64);

pub struct ResourceMapper {
    pub resources: HashMap<RetGuiResourceId, RendererResourceId>,
}

impl ResourceMapper {
    pub fn new() -> Self {
        Self {
            resources: HashMap::with_capacity(20),
        }
    }

    pub fn get(&self, resource_id: &RetGuiResourceId) -> Option<RendererResourceId> {
        self.resources.get(resource_id).cloned()
    }

    pub fn add_mapping(&mut self, retgui_resource_id: RetGuiResourceId, renderer_resource_id: RendererResourceId) {
        self.resources.insert(retgui_resource_id, renderer_resource_id);
    }

    pub fn get_all_renderer_resource_ids(&self) -> Values<'_, ResourceId, RendererResourceId> {
        self.resources.values()
    }
}

impl Default for ResourceMapper {
    fn default() -> Self {
        Self::new()
    }
}
