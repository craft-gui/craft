use crate::components::component::ComponentSpecification;
use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::events::{clicked, clicked_oku_message, OkuMessage};
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::{Display, FlexDirection, Style};
use crate::{generate_component_methods_with_generic_push, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use peniko::color::palette;
use taffy::{NodeId, Overflow, Position, TaffyTree};
use winit::event::{ButtonSource, ElementState as WinitElementState, MouseButton, MouseScrollDelta, PointerSource};
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::elements::Container;

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Dropdown {
    pub common_element_data: CommonElementData,
}

#[derive(Clone, Copy, Default)]
pub struct DropdownState {
    is_expanded: bool,
}

impl Dropdown {

    pub fn new_with_header_and_default_container(header: Container) -> Self {
        let dropdown = Dropdown {
            common_element_data: Default::default(),
        };

        dropdown
            .push(header)
            .push(
                Container::new()
                    .overlay(true)
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .background(palette::css::RED)
                    .min_width("200px")
                    .min_height("200px")
                    .position(Position::Absolute)
                    .inset("100%", "0%", "0%", "0%")
            )
            .position(Position::Relative)
    }

    pub fn new_with_header_and_container(header: Container, container: Container) -> Self {
        let dropdown = Dropdown {
            common_element_data: Default::default(),
        };

        dropdown
            .push(header)
            .push(container.overlay(true).position(Position::Absolute))
    }

    pub fn push_item<T>(mut self, component_specification: T) -> Self
    where
        T: Into<ComponentSpecification> {
        let ced = self.common_element_data_mut();

        if let Some(last) = ced.child_specs.last_mut() {
            *last = last.clone().push(component_specification.into());
        }
        self
    }

}

impl Element for Dropdown {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Dropdown"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
    ) {
        self.try_start_overlay(renderer);
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer);
        #[cfg(feature = "wgpu_renderer")]
        renderer.draw_rect(self.common_element_data.computed_layered_rectangle_transformed.padding_rectangle(), self.style().background());
       
        self.try_start_layer(renderer);
        {
            let base_state = self.get_base_state(element_state);
            let dropdown_state = base_state.data.as_ref().downcast_ref::<DropdownState>().unwrap();
            
            if dropdown_state.is_expanded {
                self.draw_children(renderer, font_system, taffy_tree, element_state, pointer);
            } else {
                self.draw_first_child(renderer, font_system, taffy_tree, element_state, pointer);
            }
        }
        self.try_end_layer(renderer);

        self.try_end_overlay(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_system: &mut FontSystem,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, font_system, element_state, scale_factor);
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
        font_system: &mut FontSystem,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
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
                font_system,
                element_state,
                pointer,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore, _font_system: &mut FontSystem) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let dropdown_state = base_state.data.as_mut().downcast_mut::<DropdownState>().unwrap();

        if clicked_oku_message(&message) {
            dropdown_state.is_expanded = !dropdown_state.is_expanded;
        }
        
        
        UpdateResult::new()
    }

    fn initialize_state(&self, _font_system: &mut FontSystem) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DropdownState::default())
        }
    }
}

impl Dropdown {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a DropdownState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut DropdownState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }

    crate::generate_component_methods_with_private_push!();
}

impl ElementStyles for Dropdown {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
