use crate::app::TAFFY_TREE;
use crate::elements::ElementInternals;
use crate::layout::TaffyTree;
use crate::text::text_context::TextContext;

use craft_primitives::geometry::{Affine, Point, Rectangle};
use craft_renderer::RenderList;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

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
    });
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
        || position != element.element_data_mut().layout.position;
    element.element_data_mut().layout.has_new_layout = has_new_layout;
    if dirty {
        element.resolve_box(position, transform, layout, z_index);
        element.apply_borders(scale_factor);
        // For scroll changes from taffy;
        element.element_data_mut().apply_scroll(layout);
        element.apply_clip(clip_bounds);
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
        false,
    )
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
        || position != element.element_data_mut().layout.position;
    element.element_data_mut().layout.has_new_layout = has_new_layout;
    if dirty {
        element.resolve_box(position, transform, layout, z_index);
        element.apply_borders(scale_factor);
        element.apply_clip(clip_bounds);
        element.element_data_mut().layout.scroll_state.mark_old();
    }

    if has_new_layout {
        taffy_tree.mark_seen(node);
    }
}

pub fn draw_generic_container(
    element: &mut dyn ElementInternals,
    renderer: &mut RenderList,
    text_context: &mut TextContext,
    scale_factor: f64,
) {
    if !element.is_visible() {
        return;
    }
    element.add_hit_testable(renderer, true, scale_factor);
    element.draw_borders(renderer, scale_factor);
    element.maybe_start_layer(renderer, scale_factor);
    element.draw_children(renderer, text_context, scale_factor);
    element.maybe_end_layer(renderer);
    element.draw_scrollbar(renderer, scale_factor);
}
