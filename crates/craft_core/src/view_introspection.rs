use crate::elements::{Font, Image, TinyVg};
use crate::events::internal::InternalMessage;
use crate::reactive::fiber_tree::FiberNode;
use craft_resource_manager::resource_type::ResourceType;
use craft_resource_manager::{ResourceIdentifier, ResourceManager};
use craft_runtime::Sender;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Introspect the view.
///
// Scans through the component tree and diffs it for resources that need to be updated.
pub fn scan_view_for_resources(
    app_sender: Sender<InternalMessage>,
    fiber_tree: Rc<RefCell<FiberNode>>,
    resource_manager: Arc<ResourceManager>,
    resources_collected: &mut HashMap<ResourceIdentifier, bool>,
) {
    let mut nodes: Vec<Rc<RefCell<FiberNode>>> = Vec::new();
    let mut to_visit: Vec<Rc<RefCell<FiberNode>>> = vec![Rc::clone(&fiber_tree)];

    while let Some(node) = to_visit.pop() {
        if node.borrow().element.is_some() {
            nodes.push(Rc::clone(&node));
        }
        for child in node.borrow().children.iter().rev() {
            to_visit.push(Rc::clone(&child));
        }
    }

    for fiber_node in nodes {
        if let Some(element) = fiber_node.borrow().element {
            let image_resource =
                element.as_any().downcast_ref::<Image>().map(|image| image.resource_identifier.clone());

            let font_resource = element.as_any().downcast_ref::<Font>().map(|font| font.resource_identifier.clone());
            let tinyvg_resource =
                element.as_any().downcast_ref::<TinyVg>().map(|tinyvg| tinyvg.resource_identifier.clone());

            if image_resource.is_some() || font_resource.is_some() || tinyvg_resource.is_some() {
                if let Some(image_resource) = image_resource {
                    resource_manager.async_download_resource_and_send_message_on_finish(
                        app_sender.clone(),
                        image_resource.clone(),
                        ResourceType::Image,
                        resources_collected,
                    );
                    resources_collected.insert(image_resource.clone(), true);
                }

                if let Some(font_resource) = font_resource {
                    resource_manager.async_download_resource_and_send_message_on_finish(
                        app_sender.clone(),
                        font_resource.clone(),
                        ResourceType::Font,
                        resources_collected,
                    );
                    resources_collected.insert(font_resource.clone(), true);
                }

                if let Some(tinyvg_resource) = tinyvg_resource {
                    resource_manager.async_download_resource_and_send_message_on_finish(
                        app_sender.clone(),
                        tinyvg_resource.clone(),
                        ResourceType::TinyVg,
                        resources_collected,
                    );
                    resources_collected.insert(tinyvg_resource.clone(), true);
                }
            }
        }
    }
}
