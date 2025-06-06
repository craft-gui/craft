use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::generate_component_methods;
use crate::geometry::{Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

/// An element for storing related elements.
#[derive(Clone, Default)]
pub struct Overlay {
    pub element_data: ElementData,
}

#[derive(Clone, Copy, Default)]
pub struct OverlayState {}

impl Element for Overlay {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Overlay"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        renderer.start_overlay();

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, element_state, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        {
            self.draw_children(renderer, text_context, element_state, pointer, window, scale_factor);
        }
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, scale_factor);

        renderer.end_overlay();
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();

        for child in self.element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
            self.element_data.layout_item.push_child(&child_node);
        }

        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree(taffy_tree, style)
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);
        self.finalize_borders(element_state);

        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.element_data().layout_item.taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                text_context,
                None,
            );
        }
    }

    fn resolve_clip(&mut self, _clip_bounds: Option<Rectangle>) {
        self.element_data.layout_item.clip_bounds = None;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(OverlayState::default()),
        }
    }
}

impl Overlay {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a OverlayState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Overlay {
        Overlay {
            element_data: Default::default(),
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Overlay {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
