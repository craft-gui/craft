use crate::components::component::{ComponentOrElement, ComponentSpecification};
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::events::OkuMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::{Display, FlexDirection, Style, Unit};
use crate::{generate_component_methods, RendererBox};
use parley::FontContext;
use std::any::Any;
use std::ops::Add;
use taffy::{Layout, NodeId, Position, TaffyTree, TraversePartialTree};
use crate::elements::{Container, Text};

/// An element for storing related elements.
#[derive(Clone, Default, Debug)]
pub struct Dropdown {
    pub common_element_data: CommonElementData,
    dropdown_selection: Option<ElementBox>,
}

#[derive(Clone, Copy, Default)]
pub struct DropdownState {
    is_open: bool,
    selected_item: Option<usize>,
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
        font_context: &mut FontContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
    ) {
        let state = self.get_state(element_state);
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer);
        self.maybe_start_layer(renderer);
        {
            let dropdown_selection = self.dropdown_selection.as_mut().unwrap();
            dropdown_selection.internal.draw(renderer, font_context, taffy_tree, dropdown_selection.internal.common_element_data().taffy_node_id.unwrap(), element_state, pointer);
            
            if state.is_open {
                self.draw_children(renderer, font_context, taffy_tree, element_state, pointer);
            }
        }
        self.maybe_end_layer(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let state = self.get_state(element_state);
        let is_open = state.is_open;
        let mut child_nodes: Vec<NodeId> = Vec::new();
        
        let selected_child =  if let Some(selected_index) = state.selected_item {
            if let Some(selected_element) = self.children_mut().get(selected_index) {
                selected_element.clone()
            } else {
                self.children_mut().first().unwrap().clone()
            }
        } else {
            self.children_mut().first().unwrap().clone()
        };
        self.dropdown_selection = Some(selected_child);
        let selected_node = self.dropdown_selection.as_mut().unwrap().internal.compute_layout(taffy_tree, element_state, scale_factor).unwrap();

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);
        
        child_nodes.push(selected_node);
        if is_open {
            let mut dropdown_list_child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());
            for child in self.children_mut().iter_mut() {
                let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
                if let Some(child_node) = child_node {
                    dropdown_list_child_nodes.push(child_node);
                }
            }
            
            let mut dropdown_list_style = Style::default();
            *dropdown_list_style.display_mut() = Display::Flex;
            *dropdown_list_style.flex_direction_mut() = FlexDirection::Column;
            *dropdown_list_style.position_mut() = Position::Absolute;
            *dropdown_list_style.inset_mut() = [Unit::Percentage(100.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0)];
            let dropdown_list_node_id = taffy_tree.new_with_children(dropdown_list_style.to_taffy_style_with_scale_factor(scale_factor), &dropdown_list_child_nodes).unwrap();
            child_nodes.push(dropdown_list_node_id);
        }
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
    ) {
        let state = self.get_state(element_state);
        let is_open = state.is_open;
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();
        
        let dropdown_selection_taffy = self.dropdown_selection.as_mut().unwrap().internal.common_element_data().taffy_node_id.unwrap();
        self.dropdown_selection.as_mut().unwrap().internal.finalize_layout(
            taffy_tree,
            dropdown_selection_taffy,
            self.common_element_data.computed_layered_rectangle.position,
            z_index,
            transform,
            element_state,
            pointer,
        );
        
        if is_open {
            let dropdown_list = taffy_tree.get_child_id(self.common_element_data.taffy_node_id.unwrap(), 1);
            let dropdown_list_result = taffy_tree.layout(dropdown_list).unwrap();
            let mut dropdown_list_starting_point = dropdown_list_result.location;
            dropdown_list_starting_point.x += self.common_element_data.computed_layered_rectangle.position.x;
            dropdown_list_starting_point.y += self.common_element_data.computed_layered_rectangle.position.y;

            for child in self.common_element_data.children.iter_mut() {
                let taffy_child_node_id = child.internal.common_element_data().taffy_node_id;
                if taffy_child_node_id.is_none() {
                    continue;
                }

                child.internal.finalize_layout(
                    taffy_tree,
                    taffy_child_node_id.unwrap(),
                    dropdown_list_starting_point.into(),
                    z_index,
                    transform,
                    element_state,
                    pointer,
                );
            }   
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn in_bounds(&self, point: Point) -> bool {
        // Check the bounds of the dropdown selection.
        let common_element_data = self.common_element_data();
        let transformed_border_rectangle = common_element_data.computed_layered_rectangle_transformed.border_rectangle();
        let dropdown_selection_in_bounds = transformed_border_rectangle.contains(&point);

        let mut dropdown_list_in_bounds = false;
        for child in self.children() {
            if child.in_bounds(point) {
                dropdown_list_in_bounds = true;
                break;
            }
        }

        dropdown_selection_in_bounds || dropdown_list_in_bounds
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<DropdownState>().unwrap();

        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {

                if message.clicked() {

                    for child in self.children().iter().enumerate() {
                        if child.1.in_bounds(pointer_button.position) {
                            state.selected_item = Some(child.0);
                            state.is_open = false;
                            break;
                        }
                    }

                    let common_element_data = self.common_element_data();
                    let transformed_border_rectangle = common_element_data.computed_layered_rectangle_transformed.border_rectangle();
                    let dropdown_selection_in_bounds = transformed_border_rectangle.contains(&pointer_button.position);
                    if dropdown_selection_in_bounds {
                        state.is_open = !state.is_open;
                    }
                    
                }

            }
            OkuMessage::Initialized => {}
            OkuMessage::KeyboardInputEvent(_) => {}
            OkuMessage::PointerMovedEvent(_) => {}
            OkuMessage::MouseWheelEvent(_) => {}
            OkuMessage::ImeEvent(_) => {}
            OkuMessage::TextInputChanged(_) => {}
        }

        UpdateResult::default()
    }

    fn initialize_state(&self) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DropdownState::default()),
        }
    }

    fn children(&self) -> Vec<&dyn Element> {
        self.common_element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }
}

impl Dropdown {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a DropdownState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Dropdown {
        Dropdown {
            common_element_data: Default::default(),
            dropdown_selection: Default::default(),
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Dropdown {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
