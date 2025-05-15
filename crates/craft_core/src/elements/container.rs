use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::Event;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::layout::layout_context::LayoutContext;
use crate::elements::scroll_state::ScrollState;
use crate::events::CraftMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods};
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::renderer::renderer::RenderList;
use crate::text::text_context::TextContext;

/// An element for storing related elements.
#[derive(Clone, Default)]
pub struct Container {
    pub element_data: ElementData,
}

#[derive(Clone, Copy, Default)]
pub struct ContainerState {
    pub(crate) scroll_state: ScrollState,
}

impl Element for Container {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Container"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        let base_state = self.get_base_state_mut(element_state);
        let current_style = base_state.base.current_style(self.element_data());
        
        if !current_style.visible() {
            return;
        }
        
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, element_state);
        self.maybe_start_layer(renderer);
        {
            self.draw_children(renderer, text_context, taffy_tree, element_state, pointer, window);
        }
        self.maybe_end_layer(renderer);

        self.draw_scrollbar(renderer);
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

        let base_state = self.get_base_state_mut(element_state);
        let current_style = base_state.base.current_style(self.element_data());
        let style: taffy::Style = current_style.to_taffy_style_with_scale_factor(scale_factor);

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
        text_context: &mut TextContext,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders(element_state);

        self.element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.element_data.component_id).unwrap().data.downcast_ref::<ContainerState>()
        {
            container_state.scroll_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);
        let child_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, -scroll_y, 0.0));

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
                transform * child_transform,
                element_state,
                pointer,
                text_context,
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
        _text_context: &mut TextContext,
        should_style: bool,
    ) -> Event {
        self.on_style_event(message, element_state, should_style);
        let base_state = self.get_base_state_mut(element_state);
        let container_state = base_state.data.as_mut().downcast_mut::<ContainerState>().unwrap();

        container_state.scroll_state.on_event(message, &self.element_data, &mut base_state.base)
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(ContainerState::default()),
        }
    }
}

impl Container {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a ContainerState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Container {
        Container {
            element_data: Default::default(),
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Container {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
