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
use crate::style::{Display, FlexDirection, Style};
use crate::{generate_component_methods_no_children, RendererBox};
use parley::FontContext;
use std::any::Any;
use taffy::{NodeId, Position, TaffyTree};
use crate::elements::{Container, Text};

/// An element for storing related elements.
#[derive(Clone, Default, Debug)]
pub struct Dropdown {
    pub common_element_data: CommonElementData,
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
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer);
        self.maybe_start_layer(renderer);
        {
            self.draw_children(renderer, font_context, taffy_tree, element_state, pointer);
        }
        self.maybe_end_layer(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        let state = self.get_base_state_mut(element_state).data.as_mut().downcast_mut::<DropdownState>().unwrap();

        // Update the dropdown selection based off the current selection.
        let mut selection: Option<ElementBox> = None;
        if let Some(dropdown_list) = self.dropdown_list_mut() {
            for child in dropdown_list.internal.children_mut().iter_mut().enumerate() {
                if Some(child.0) == state.selected_item {
                    selection = Some(child.1).cloned()
                }
            }
        }

        if let Some(dropdown_selection) = self.dropdown_selection_mut() {
            if let Some(selection) = selection {
                *dropdown_selection = selection;
            }
        }

        if state.is_open {
            for child in self.children_mut().iter_mut() {
                let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
                if let Some(child_node) = child_node {
                    child_nodes.push(child_node);
                }
            }
        } else if let Some(dropdown_selection) = self.dropdown_selection_mut() {
            let child_node = dropdown_selection.internal.compute_layout(taffy_tree, element_state, scale_factor);
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
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);

        self.finalize_borders();

        self.common_element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.common_element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

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
            );
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

        // Check the bounds of the dropdown list.
        let dropdown_list_in_bounds = if let Some(dropdown_list) = self.dropdown_list() {
            dropdown_list.in_bounds(point)
        } else {
            false
        };

        dropdown_selection_in_bounds || dropdown_list_in_bounds
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<DropdownState>().unwrap();

        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {

                if message.clicked() {

                    if let Some(dropdown_list) = self.dropdown_list() {
                        for dropdown_list_item in dropdown_list.children().iter().enumerate() {
                            if dropdown_list_item.1.in_bounds(pointer_button.position) {
                                state.selected_item = Some(dropdown_list_item.0);
                                state.is_open = !state.is_open;
                            }
                        }
                    }

                    if let Some(child) = self.dropdown_selection() {
                        if child.in_bounds(pointer_button.position) {
                            state.is_open = !state.is_open;
                        }
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
        }
    }

    // Note: Only use these functions in update, on_event, but not in user-land functions like .push().
    pub(crate) fn dropdown_selection(&self) -> Option<&dyn Element> {
        self.children().first().map(|v| &**v)
    }

    pub(crate) fn dropdown_list(&self) -> Option<&dyn Element> {
        self.children().get(1).map(|v| &**v)
    }

    pub(crate) fn dropdown_selection_mut(&mut self) -> Option<&mut ElementBox> {
        self.children_mut().first_mut()
    }

    pub(crate) fn dropdown_list_mut(&mut self) -> Option<&mut ElementBox> {
        self.children_mut().get_mut(1)
    }

    generate_component_methods_no_children!();

    #[allow(dead_code)]
    pub fn push<T>(mut self, component_specification: T) -> Self
    where
        T: Into<ComponentSpecification> + Clone,
    {

        if self.common_element_data.child_specs.is_empty() {
            // Create the selection when pushing to the 1st element.
            self.common_element_data.child_specs.push(Container::new().push(component_specification.clone()).into());

            // Create the dropdown list with their 1st child.
            self.common_element_data.child_specs.push(
                Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .position(Position::Absolute)
                    .inset("100%", "0%", "0%", "0%")
                    .push(component_specification.clone())
                    .into()
            );
        } else if let Some(dropdown_list) = self.common_element_data.child_specs.get_mut(1) {
            // Append to the dropdown list.
            dropdown_list.children.push(component_specification.into());
        }

        self
    }
}

impl ElementStyles for Dropdown {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
