use crate::resource::Resource;
use crate::resource_type::ResourceType;
use crate::ResourceIdentifier;

#[derive(Debug)]
pub enum ResourceEvent {
    Loaded(ResourceIdentifier, ResourceType, Resource),
    #[allow(dead_code)]
    UnLoaded(ResourceIdentifier),
}
