pub use crate::identifier::ResourceId;

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use retgui_runtime::RetGuiRuntimeHandle;

use crate::decoders::{image_decoder, tinyvg_decoder};
use crate::lock_free_map::LockFreeMap;
use crate::resource::Resource;
use crate::resource_type::ResourceType;

pub mod decoders;
pub mod image;
pub mod resource;
pub mod resource_event;
pub mod resource_type;

mod identifier;
mod lock_free_map;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;
pub type DecoderFn = fn(Vec<u8>) -> Box<dyn Any + Send + Sync>;

pub struct ResourceManager {
    resources: LockFreeMap<ResourceId, Resource>,
    pub(crate) runtime: RetGuiRuntimeHandle,
    decoders: HashMap<ResourceType, DecoderFn>,
}

impl ResourceManager {
    pub fn new(retgui_runtime_handle: RetGuiRuntimeHandle) -> Self {
        Self {
            resources: LockFreeMap::new(),
            runtime: retgui_runtime_handle,
            decoders: HashMap::from([
                (
                    ResourceType::Image,
                    image_decoder as fn(Vec<u8>) -> Box<dyn Any + Send + Sync>,
                ),
                (
                    ResourceType::TinyVg,
                    tinyvg_decoder as fn(Vec<u8>) -> Box<dyn Any + Send + Sync>,
                ),
            ]),
        }
    }

    pub fn async_download_resource(self: &Arc<Self>, resource_id: ResourceId, resource_type: &ResourceType) {
        let resource_manager = self.clone();
        let resource_type = resource_type.clone();
        let decoder_fn = *self.decoders.get(&resource_type).unwrap();
        let f = async move {
            let bytes = resource_id.fetch_data_from_resource_id().await;

            let resource = Resource {
                resource_type,
                data: decoder_fn(bytes.unwrap()),
                expiration_time: None,
            };

            resource_manager.insert(resource_id, Arc::new(resource));
        };

        self.runtime.spawn(f);
    }

    pub fn contains(&self, resource_id: &ResourceId) -> bool {
        self.resources.contains(resource_id)
    }

    pub fn get(&self, resource_id: &ResourceId) -> Option<Arc<Resource>> {
        self.resources.get(resource_id)
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Arc<Resource>) {
        self.resources.insert(resource_id, resource);
    }
}
