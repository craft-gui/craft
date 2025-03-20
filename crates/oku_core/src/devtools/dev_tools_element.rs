use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::{ComponentId, UpdateResult};
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::style::Style;
use crate::{generate_component_methods, RendererBox};
use parley::FontContext;
use std::any::Any;
use peniko::Brush;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct DevTools {
    pub common_element_data: CommonElementData,
    /// The tree to inspect.
    pub(crate) debug_inspector_tree: Option<Box<dyn Element>>,
    /// The selected element in the inspector tree.
    pub(crate) selected_inspector_element: Option<ComponentId>,
    /// The hovered element in the inspector tree.
    pub(crate) hovered_inspector_element: Option<ComponentId>,
}

#[derive(Clone, Copy, Default)]
pub struct DevToolsState {}

impl DevTools {
    pub fn push_debug_inspector_tree(mut self, root: Box<dyn Element>) -> Self {
        self.debug_inspector_tree = Some(root.clone());
        self
    }
    pub fn push_selected_inspector_element(mut self, element: Option<ComponentId>) -> Self {
        self.selected_inspector_element = element;
        self
    }
    pub fn push_hovered_inspector_element(mut self, element: Option<ComponentId>) -> Self {
        self.hovered_inspector_element = element;
        self
    }
}

impl Element for DevTools {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Dev Tools"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_context: &mut FontContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
    ) {
        self.draw_borders(renderer);
        self.draw_children(renderer, font_context, taffy_tree, element_state, pointer);

        // Find the element we are hovering over and draw an overlay.
        if let Some(hovered_inspector_element_component_id) = self.hovered_inspector_element {
            let mut hovered_inspector_element: Option<&dyn Element> = None;
            let root = self.debug_inspector_tree.as_ref().unwrap();

            // Find the hovered inspector element.
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != hovered_inspector_element_component_id {
                    continue;
                }

                hovered_inspector_element = Some(*Box::new(<&dyn Element>::clone(element)));
                break;
            }

            // Highlight the hovered element and draw their margin, padding, and content box.
            if let Some(selected_element) = hovered_inspector_element {
                // FIXME: Make use of layers, so that the boxes only mix with the element's colors.
                let content_box_highlight_color = Color::from_rgba8(184, 226, 243, 125);
                let padding_box_highlight_color = Color::from_rgba8(102, 87, 166, 125);
                let margin_box_highlight_color = Color::from_rgba8(115, 118, 240, 50);

                let margin_rectangle =
                    selected_element.common_element_data().computed_layered_rectangle_transformed.margin_rectangle();
                renderer.push_layer(margin_rectangle);
                renderer.draw_rect(margin_rectangle, margin_box_highlight_color);
                renderer.pop_layer();

                let padding_rectangle =
                    selected_element.common_element_data().computed_layered_rectangle_transformed.padding_rectangle();
                renderer.push_layer(padding_rectangle);
                renderer.draw_rect(padding_rectangle, padding_box_highlight_color);
                renderer.pop_layer();

                let content_rectangle =
                    selected_element.common_element_data().computed_layered_rectangle_transformed.content_rectangle();
                renderer.push_layer(content_rectangle);
                renderer.draw_rect(content_rectangle, content_box_highlight_color);
                renderer.pop_layer();
            }
        }
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
            if let Some(child_node) = child_node {
                child_nodes.push(child_node);
            }
        }

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &child_nodes).unwrap());
        self.common_element_data().taffy_node_id
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
        font_context: &mut FontContext,
        layout_context: &mut parley::LayoutContext<Brush>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);

        self.finalize_borders();

        for child in self.common_element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.common_element_data().taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.common_element_data.computed_layered_rectangle.position,
                z_index,
                transform,
                element_state,
                pointer,
                font_context,
                layout_context
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, _message: OkuMessage, element_state: &mut ElementStateStore) -> UpdateResult {
        let _dev_tools_state = self.get_state_mut(element_state);

        UpdateResult::default()
    }

    fn initialize_state(&self) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DevToolsState::default()),
        }
    }
}

impl DevTools {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a DevToolsState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut DevToolsState {
        element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap()
    }

    pub fn new() -> DevTools {
        DevTools {
            debug_inspector_tree: None,
            common_element_data: Default::default(),
            selected_inspector_element: None,
            hovered_inspector_element: None,
        }
    }

    generate_component_methods!();
}

impl ElementStyles for DevTools {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}
