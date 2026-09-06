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
#[derive(Debug)]
pub struct ResourceError {
    pub resource_type: ResourceType,
    pub message: String,
}

impl ResourceError {
    pub fn new(resource_type: ResourceType, message: impl Into<String>) -> Self {
        Self {
            resource_type,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.resource_type, self.message)
    }
}

impl std::error::Error for ResourceError {}

pub type DecoderFn = fn(Vec<u8>) -> Result<Box<dyn Any + Send + Sync>, ResourceError>;

pub struct ResourceManager {
    resources: LockFreeMap<ResourceId, Resource>,
    decoders: HashMap<ResourceType, DecoderFn>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: LockFreeMap::new(),
            decoders: HashMap::from([
                (ResourceType::Image, image_decoder as DecoderFn),
                (ResourceType::TinyVg, tinyvg_decoder as DecoderFn),
            ]),
        }
    }

    pub fn upload(
        &self,
        resource_id: ResourceId,
        resource_type: ResourceType,
        bytes: Vec<u8>,
    ) -> Result<(), ResourceError> {
        let data: Box<dyn Any + Send + Sync> = if matches!(resource_type, ResourceType::Other(_)) {
            Box::new(bytes)
        } else {
            let decoder = self
                .decoders
                .get(&resource_type)
                .ok_or_else(|| ResourceError::new(resource_type.clone(), "No decoder registered"))?;
            decoder(bytes)?
        };
        self.insert(
            resource_id,
            Arc::new(Resource {
                resource_type,
                data,
                expiration_time: None,
            }),
        );
        Ok(())
    }

    pub fn async_download_resource(
        self: &Arc<Self>,
        resource_id: ResourceId,
        resource_type: &ResourceType,
        runtime: &RetGuiRuntimeHandle,
    ) {
        let resource_manager = self.clone();
        let resource_type = resource_type.clone();
        let f = async move {
            if let Some(bytes) = resource_id.fetch_data_from_resource_id().await {
                if let Err(error) = resource_manager.upload(resource_id.clone(), resource_type, bytes) {
                    retgui_logging::error!("Failed to decode {resource_id}: {error}");
                }
            } else {
                retgui_logging::error!("Failed to load {resource_id}");
            }
        };

        runtime.spawn(f);
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
