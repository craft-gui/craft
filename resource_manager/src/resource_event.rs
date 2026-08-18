use crate::ResourceId;
use crate::resource::Resource;
use crate::resource_type::ResourceType;

#[derive(Debug)]
pub enum ResourceEvent {
    Loaded(ResourceId, ResourceType, Resource),
    #[allow(dead_code)]
    UnLoaded(ResourceId),
}
