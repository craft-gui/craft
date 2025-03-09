use crate::elements::element::Element;
use crate::elements::{Font, Image};
use crate::reactive::fiber_node::FiberNode;
use crate::reactive::tree::ComponentTreeNode;
use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::{ResourceManager};

use parley::FontContext;

use tokio::sync::RwLock;

use std::sync::{Arc};

/// Introspect the view.

// Scans through the component tree and diffs it for resources that need to be updated.
pub async fn scan_view_for_resources(element: &dyn Element, component: &ComponentTreeNode, resource_manager: Arc<RwLock<ResourceManager>>, font_context: &mut FontContext) {
    let fiber: FiberNode = FiberNode {
        element: Some(element),
        component: Some(component),
    };
    
    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            let image_resource = element
                .as_any()
                .downcast_ref::<Image>()
                .map(|image| image.resource_identifier.clone());

            let font_resource = element
                .as_any()
                .downcast_ref::<Font>()
                .map(|font| font.resource_identifier.clone());

            if image_resource.is_some() || font_resource.is_some() {
                let mut resource_manager = resource_manager.write().await;
                
                
                if let Some(image_resource) = image_resource {
                    resource_manager.async_download_resource_and_send_message_on_finish(image_resource.clone(), ResourceType::Image);
                    resource_manager.add_temporary_resource(image_resource.clone(), ResourceType::Image);
                }
                
                if let Some(font_resource) = font_resource {
                    resource_manager.async_download_resource_and_send_message_on_finish(font_resource.clone(), ResourceType::Font);
                    resource_manager.add_temporary_resource(font_resource.clone(), ResourceType::Font);
                }
            }
            
        }
    }
}