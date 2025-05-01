use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

/// An element for storing related elements.
#[derive(Clone, Default, Debug)]
pub struct Overlay {
    pub element_data: ElementData,
}

#[derive(Clone, Copy, Default)]
pub struct OverlayState {
}

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
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        renderer.start_overlay();
        
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer);
        self.maybe_start_layer(renderer);
        {
            self.draw_children(renderer, font_system, taffy_tree, element_state, pointer, window);
        }
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer);
        
        renderer.end_overlay();
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
            if let Some(child_node) = child_node {
                child_nodes.push(child_node);
            }
        }

        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &child_nodes).unwrap());
        self.element_data().taffy_node_id
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
        font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders();
        
        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.element_data().taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                font_system,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _font_system: &mut FontSystem,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    fn initialize_state(&self, _font_system: &mut FontSystem, _scaling_factor: f64) -> ElementStateStoreItem {
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
