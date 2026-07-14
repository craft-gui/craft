use std::collections::HashMap;
use std::collections::hash_map::Values;
use craft_resource_manager::{ResourceId as CraftResourceId, ResourceId};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RendererResourceId(pub u64);

pub struct ResourceMapper {
    pub resources: HashMap<CraftResourceId, RendererResourceId>,
}

impl ResourceMapper {
    pub fn new() -> Self {
        Self {
            resources: HashMap::with_capacity(20),
        }
    }

    pub fn get(&self, resource_id: &CraftResourceId) -> Option<RendererResourceId> {
        self.resources.get(resource_id).cloned()
    }

    pub fn add_mapping(&mut self, craft_resource_id: CraftResourceId, renderer_resource_id: RendererResourceId) {
        self.resources.insert(craft_resource_id, renderer_resource_id);
    }

    pub fn get_all_renderer_resource_ids(&self) -> Values<'_, ResourceId, RendererResourceId> {
        self.resources.values()
    }
}