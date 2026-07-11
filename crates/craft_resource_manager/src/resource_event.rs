use crate::ResourceId;
use crate::resource::Resource;

#[derive(Debug)]
pub enum ResourceEvent {
    Loaded(ResourceId, String, Resource),
    #[allow(dead_code)]
    UnLoaded(ResourceId),
}
