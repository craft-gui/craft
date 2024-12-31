use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::ResourceIdentifier;

#[derive(Debug)]
pub enum ResourceEvent {
    Added((ResourceIdentifier, ResourceType)),
    Loaded(ResourceIdentifier),
    UnLoaded(ResourceIdentifier),
}
