use crate::elements::{DynElement, ElementInternals, Elements};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

use crate::elements::element_data::ElementData;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;
use std::sync::Arc;

/// A helper to push children.
pub fn push_child_to_element(elements: &mut Elements, parent_handle: DynElement, child: DynElement) {
    if !elements.contains(parent_handle) || !elements.contains(child) {
        return;
    }
    let (me, me_window, redraw_signal, scale_factor, parent_id, access) = {
        let parent = elements.get(parent_handle);
        let data = parent.element_data();
        (
            data.me,
            data.window,
            data.redraw_signal.clone(),
            data.applied_scale_factor,
            parent.child_layout_parent(),
            data.access_key
                .zip(data.access_root)
                .map(|nodes| (data.access_tree.clone(), nodes)),
        )
    };
    elements.dispatch_mut(child, |child_element, elements| {
        child_element.element_data_mut().parent = Some(me);
        child_element.element_data_mut().window = me_window;
        child_element.element_data_mut().redraw_signal = redraw_signal;
        child_element.propagate_window_down(elements);
        child_element.set_scale_factor(elements, scale_factor);
        elements.get_mut(parent_handle).element_data_mut().children.push(child);

        if let Some(child_id) = child_element.element_data().layout.gummy_node_id {
            elements.gummy_tree.add_child(parent_id.unwrap(), child_id);
        }
        child_element.on_post_add_layout_tree(&mut elements.gummy_tree);

        if let Some((tree, (parent_node, root))) = access {
            crate::accessibility::reparent_subtree(elements, child_element, &tree, parent_node, root, scale_factor);
        }
    });
    elements.get(parent_handle).request_window_redraw();
}

pub fn apply_generic_container_layout(
    element: &mut dyn ElementInternals,
    gummy_tree: &mut GummyTree,
    z_index: &mut u32,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

    element.element_data_mut().layout.has_new_layout.set(has_new_layout);
    if has_new_layout {
        element.resolve_box(layout, z_index);
        element.apply_borders(scale_factor);
        // For scroll changes from gummy;
        element.element_data_mut().apply_scroll(layout);
    }

    // For manual scroll updates.
    if !has_new_layout && element.element_data_mut().layout.scroll_state.is_new() {
        element.element_data_mut().apply_scroll(layout);
        element.element_data_mut().layout.scroll_state.mark_old();
    }

    if has_new_layout {
        gummy_tree.mark_seen(node);
    }
}

pub fn apply_generic_container_layout_non_dom(
    element: &mut ElementData,
    gummy_tree: &mut GummyTree,
    z_index: &mut u32,
    scale_factor: f64,
) {
    let node = element.layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

    element.layout.has_new_layout.set(has_new_layout);
    if has_new_layout {
        element.layout.resolve_box(layout, z_index);
        element.apply_borders(scale_factor);
        // For scroll changes from gummy;
        element.apply_scroll(layout);
    }

    // For manual scroll updates.
    if !has_new_layout && element.layout.scroll_state.is_new() {
        element.apply_scroll(layout);
        element.layout.scroll_state.mark_old();
    }

    if has_new_layout {
        gummy_tree.mark_seen(node);
    }
}

pub fn apply_generic_leaf_layout(
    element: &mut dyn ElementInternals,
    gummy_tree: &mut GummyTree,
    z_index: &mut u32,
    scale_factor: f64,
) {
    let node = element.element_data_mut().layout.gummy_node_id.unwrap();
    let layout = gummy_tree.get_layout(node);
    let has_new_layout = gummy_tree.has_new_layout(node);

    element.element_data_mut().layout.has_new_layout.set(has_new_layout);
    if has_new_layout {
        element.resolve_box(layout, z_index);
        element.apply_borders(scale_factor);
    }

    if has_new_layout {
        gummy_tree.mark_seen(node);
    }
}

pub fn draw_generic_container(
    element: &dyn ElementInternals,
    elements: &Elements,
    renderer: &mut dyn Renderer,
    resource_manager: Arc<ResourceManager>,
    text_context: &mut TextContext,
    scale_factor: f64,
) {
    if !element.is_visible() {
        return;
    }

    element.maybe_start_overlay(renderer);

    element.add_hit_testable(renderer, true, scale_factor);
    element.draw_borders(renderer, scale_factor);
    element.maybe_start_layer(renderer, scale_factor);
    element.draw_children(elements, renderer, resource_manager.clone(), scale_factor, text_context);
    element.maybe_end_layer(renderer);
    element.draw_scrollbar(renderer, scale_factor);

    element.maybe_end_overlay(renderer);
}
