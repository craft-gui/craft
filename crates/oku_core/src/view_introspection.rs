use std::path::PathBuf;
use crate::App;
use crate::elements::element::Element;
use crate::elements::Image;
use crate::reactive::fiber_node::FiberNode;
use crate::reactive::tree::ComponentTreeNode;
use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::ResourceIdentifier;

/// Introspect the view.

// Scans through the component tree and diffs it for resources that need to be updated.
pub async fn scan_view_for_resources(element: &dyn Element, component: &ComponentTreeNode, app: &mut App) {
    let fiber: FiberNode = FiberNode {
        element: Some(element),
        component: Some(component),
    };

    let resource_manager = &mut app.resource_manager;
    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            let image_resource = if let Some(image) = element.as_any().downcast_ref::<Image>() {
                Some(image.resource_identifier.clone())
            } else {
                None
            };

            if image_resource.is_some() {
                let mut resource_manager = resource_manager.write().await;
                
                if let Some(image_resource) = image_resource {
                    resource_manager.add(image_resource.clone(), ResourceType::Image).await;
                }
            }
            
        }
    }
}