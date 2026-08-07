use crate::app::GUMMY_TREE;
use crate::elements::ElementInternals;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

use retgui_primitives::geometry::{Affine, Point, Rectangle};

use crate::elements::element_data::ElementData;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// A helper to push children.
pub fn push_child_to_element(parent: &mut dyn ElementInternals, child: Rc<RefCell<dyn ElementInternals>>) {
    let (me, me_window) = {
        let element_data = parent.element_data();
        (element_data.me.clone(), element_data.window.clone())
    };
    child.borrow_mut().element_data_mut().parent = Some(me);
    child.borrow_mut().element_data_mut().window = me_window;
    child.borrow_mut().propagate_window_down();
    parent.element_data_mut().children.push(child.clone());

    // Add the children's gummy node.
    GUMMY_TREE.with_borrow_mut(|gummy_tree| {
        let parent_id = parent.element_data().layout.gummy_node_id.unwrap();
        let child_id = child.borrow().element_data().layout.gummy_node_id;
        if let Some(child_id) = child_id {
            gummy_tree.add_child(parent_id, child_id);
        }
        child.borrow_mut().on_post_add_layout_tree(gummy_tree);
    });

    {
        let data = parent.element_data();
        if let Some((parent_node, root)) = data.access_key.zip(data.access_root) {
            crate::accessibility::reparent_subtree(
                &mut *child.borrow_mut(),
                &data.access_tree,
                parent_node,
                root,
                data.access_scale_factor,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_generic_container_layout(
    element: &mut dyn ElementInternals,
    gummy_tree: &mut GummyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    text_context: &mut TextContext,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

    let dirty = has_new_layout
        || transform != element.element_data_mut().layout.get_transform()
        || position != element.element_data_mut().layout.position
        || clip_bounds != element.element_data().layout.parent_clip;
    element.element_data_mut().layout.has_new_layout = has_new_layout;
    if dirty {
        element.resolve_box(position, transform, layout, z_index);
        element.apply_borders(scale_factor);
        // For scroll changes from gummy;
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
        gummy_tree.mark_seen(node);
    }

    element
        .element_data_mut()
        .set_accessibility_bounds_from_layout(scale_factor);

    let scroll_y = element.element_data_mut().scroll().scroll_y() as f64;
    let child_transform = Affine::translate((0.0, -scroll_y));

    element.apply_layout_children(
        gummy_tree,
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
    gummy_tree: &mut GummyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

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
        // For scroll changes from gummy;
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
        gummy_tree.mark_seen(node);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_generic_leaf_layout(
    element: &mut dyn ElementInternals,
    gummy_tree: &mut GummyTree,
    position: Point,
    z_index: &mut u32,
    transform: Affine,
    clip_bounds: Option<Rectangle>,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

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
        gummy_tree.mark_seen(node);
    }

    element
        .element_data_mut()
        .set_accessibility_bounds_from_layout(scale_factor);
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
