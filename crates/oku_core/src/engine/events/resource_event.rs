use crate::platform::resource_manager::ResourceIdentifier;

#[derive(Debug)]
pub enum ResourceEvent {
    Added(ResourceIdentifier),
    Loaded(ResourceIdentifier),
    UnLoaded(ResourceIdentifier),
}
