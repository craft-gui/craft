use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::ResourceIdentifier;

#[derive(Debug)]
pub enum ResourceEvent {
    Loaded(ResourceIdentifier, ResourceType),
    #[allow(dead_code)]
    UnLoaded(ResourceIdentifier),
}
