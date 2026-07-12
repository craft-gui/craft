use crate::app::TAFFY_TREE;
use crate::elements::ElementInternals;
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;

use craft_primitives::geometry::{Affine, Point, Rectangle};

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use crate::elements::element_data::ElementData;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use accesskit::{Node, NodeId, TreeUpdate};
use craft_renderer::renderer::Renderer;
use craft_resource_manager::ResourceManager;

/// A helper to push children.
pub fn push_child_to_element(parent: &mut dyn ElementInternals, child: Rc<RefCell<dyn ElementInternals>>) {
    let element_data = parent.element_data_mut();
    let me: Weak<RefCell<dyn ElementInternals>> = element_data.me.clone();
    let me_window = element_data.window.clone();
    child.borrow_mut().element_data_mut().parent = Some(me);
    child.borrow_mut().element_data_mut().window = me_window;
    child.borrow_mut().propagate_window_down();
    element_data.children.push(child.clone());

    // Add the children's taffy node.
    TAFFY_TREE.with_borrow_mut(|taffy_tree| {
        let parent_id = element_data.layout.taffy_node_id.unwrap();
        let child_id = child.borrow().element_data().layout.taffy_node_id;
        if let Some(child_id) = child_id {
            taffy_tree.add_child(parent_id, child_id);
        }
        child.borrow_mut().on_post_add_layout_tree(taffy_tree);
    })
}

#[allow(clippy::too_many_arguments)]
pub fn apply_generic_container_layout(
    element: &mut dyn ElementInternals,
    taffy_tree: &mut TaffyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    text_context: &mut TextContext,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.taffy_node_id.unwrap();
    let layout = taffy_tree.get_layout(node);
    let has_new_layout = taffy_tree.has_new_layout(node);

    let dirty = has_new_layout
        || transform != element.element_data_mut().layout.get_transform()
        || position != element.element_data_mut().layout.position
        || clip_bounds != element.element_data().layout.parent_clip;
    element.element_data_mut().layout.has_new_layout = has_new_layout;
    if dirty {
        element.resolve_box(position, transform, layout, z_index);
        element.apply_borders(scale_factor);
        // For scroll changes from taffy;
        element.element_data_mut().apply_scroll(layout);
        element.apply_clip(clip_bounds);
        element.element_data_mut().layout.parent_clip = clip_bounds;
        element.element_data_mut().layout.scroll_state.mark_old();
    }

    // For manual scroll updates.
    if !dirty && element.element_data_mut().layout.scroll_state.is_new() {
        element.element_data_mut().apply_scroll(layout);
        element.element_data_mut().layout.scroll_state.mark_old();
    }

    if has_new_layout {
        taffy_tree.mark_seen(node);
    }

    let scroll_y = element.element_data_mut().scroll().scroll_y() as f64;
    let child_transform = Affine::translate((0.0, -scroll_y));

    element.apply_layout_children(
        taffy_tree,
        z_index,
        transform * child_transform,
        text_context,
        scale_factor,
        element.element_data().layout.clip_bounds,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn apply_generic_container_layout_non_dom(
    element: &mut ElementData,
    taffy_tree: &mut TaffyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.layout.taffy_node_id.unwrap();
    let layout = taffy_tree.get_layout(node);
    let has_new_layout = taffy_tree.has_new_layout(node);

    let dirty = has_new_layout
        || transform != element.layout.get_transform()
        || position != element.layout.position
        || clip_bounds != element.layout.parent_clip;
    element.layout.has_new_layout = has_new_layout;
    if dirty {
        element
            .layout
            .resolve_box(position, transform, layout, z_index, element.style.get_position());
        element.apply_borders(scale_factor);
        // For scroll changes from taffy;
        element.apply_scroll(layout);
        element.layout.apply_clip(clip_bounds);
        element.layout.parent_clip = clip_bounds;
        element.layout.scroll_state.mark_old();
    }

    // For manual scroll updates.
    if !dirty && element.layout.scroll_state.is_new() {
        element.apply_scroll(layout);
        element.layout.scroll_state.mark_old();
    }

    if has_new_layout {
        taffy_tree.mark_seen(node);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_generic_leaf_layout(
    element: &mut dyn ElementInternals,
    taffy_tree: &mut TaffyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.taffy_node_id.unwrap();
    let layout = taffy_tree.get_layout(node);
    let has_new_layout = taffy_tree.has_new_layout(node);

    let dirty = has_new_layout
        || transform != element.element_data_mut().layout.get_transform()
        || position != element.element_data_mut().layout.position
        || clip_bounds != element.element_data().layout.parent_clip;
    element.element_data_mut().layout.has_new_layout = has_new_layout;
    if dirty {
        element.resolve_box(position, transform, layout, z_index);
        element.apply_borders(scale_factor);
        element.apply_clip(clip_bounds);
        element.element_data_mut().layout.parent_clip = clip_bounds;
        element.element_data_mut().layout.scroll_state.mark_old();
    }

    if has_new_layout {
        taffy_tree.mark_seen(node);
    }
}

#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
pub fn add_generic_accesskit_data(
    element: &mut ElementData,
    mut current_node: Node,
    current_node_id: NodeId,
    tree: &mut TreeUpdate,
    parent_index: Option<usize>,
    scale_factor: f64,
) {
    let padding_box = element
        .layout
        .computed_box_transformed
        .padding_rectangle()
        .scale(scale_factor);

    current_node.set_bounds(accesskit::Rect {
        x0: padding_box.left() as f64,
        y0: padding_box.top() as f64,
        x1: padding_box.right() as f64,
        y1: padding_box.bottom() as f64,
    });

    let current_index = tree.nodes.len(); // The current node is the last one added.

    if let Some(parent_index) = parent_index {
        let parent_node = tree.nodes.get_mut(parent_index).unwrap();
        parent_node.1.push_child(current_node_id);
    }

    tree.nodes.push((current_node_id, current_node));

    for child in element.children.iter_mut() {
        child
            .borrow_mut()
            .compute_accessibility_tree(tree, Some(current_index), scale_factor);
    }
}

pub fn draw_generic_container(
    element: &mut dyn ElementInternals,
    renderer: &mut dyn Renderer,
    resource_manager: Arc<ResourceManager>,
    text_context: &mut TextContext,
    scale_factor: f64,
) {
    if !element.is_visible() {
        return;
    }
    element.add_hit_testable(renderer, true, scale_factor);
    element.draw_borders(renderer, scale_factor);
    element.maybe_start_layer(renderer, scale_factor);
    element.draw_children(renderer, resource_manager.clone(), scale_factor, text_context);
    element.maybe_end_layer(renderer);
    element.draw_scrollbar(renderer, scale_factor);
}
