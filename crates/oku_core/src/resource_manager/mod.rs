mod identifier;
mod image;
pub mod resource;
pub mod resource_data;

use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::events::resource_event::ResourceEvent;
pub use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::resource::Resource;
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub(crate) resource_jobs: VecDeque<ResourceFuture>,
    pub(crate) resources: HashMap<ResourceIdentifier, Resource>,
    pub(crate) app_sender: Sender<AppMessage>,
}

impl ResourceManager {
    pub(crate) fn new(app_sender: Sender<AppMessage>) -> Self {
        Self {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
            app_sender,
        }
    }

    pub async fn add(&mut self, resource: ResourceIdentifier) {
        if !self.resources.contains_key(&resource) {
            let image = resource.fetch_resource_from_resource_identifier().await;

            if let Some(image_resource) = image {
                let resource_copy = resource.clone();
                self.resources.insert(resource, image_resource);

                self.app_sender
                    .send(AppMessage::new(0, InternalMessage::ResourceEvent(ResourceEvent::Added(resource_copy))))
                    .await
                    .expect("Failed to send added resource event");
            }
        }
    }
}
