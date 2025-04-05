use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::element_data::ElementData;
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::elements::Container;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::{AlignItems, Display, FlexDirection, Style, Unit};
use crate::{generate_component_methods, RendererBox};
use cosmic_text::FontSystem;
use peniko::Color;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, Position, TaffyTree, TraversePartialTree};
use winit::window::Window;

/// The index of the dropdown list in the layout tree.
const DROPDOWN_LIST_INDEX: usize = 1;

/// An element for displaying a list of items in a dropdown. By default, the first list item will be shown, otherwise show the selected item. 
#[derive(Clone, Default, Debug)]
pub struct Dropdown {
    pub element_data: ElementData,
    /// A copy of the currently selected element, this is not stored in the user tree nor will it receive events.
    /// This is copied, so that we can change the location and render it in the dropdown container.
    pseudo_dropdown_selection: Option<ElementBoxed>,
    /// An element not in the user tree. Created, so that we can utilize our existing functionality (like scrollbars).
    pseudo_dropdown_list_element: Container,
}

#[derive(Clone, Copy, Default)]
pub struct DropdownState {
    /// Whether the dropdown list is visible or not.
    is_open: bool,
    /// The index of the currently selected item in the dropdown list.
    /// For example if you select the first item, `selected_item` will be 0.
    selected_item: Option<usize>,
}

impl Element for Dropdown {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn children(&self) -> Vec<&dyn Element> {
        self.element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }

    /// Checks if a point is in the dropdown selection and/or list.
    fn in_bounds(&self, point: Point) -> bool {
        // Check the bounds of the dropdown selection.
        let element_data = self.element_data();
        let transformed_border_rectangle = element_data.computed_box_transformed.border_rectangle();
        let dropdown_selection_in_bounds = transformed_border_rectangle.contains(&point);

        // Check the bounds of the dropdown list items.
        let mut dropdown_list_in_bounds = false;
        for child in self.children() {
            if child.in_bounds(point) {
                dropdown_list_in_bounds = true;
                break;
            }
        }

        dropdown_selection_in_bounds || dropdown_list_in_bounds
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
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let is_open = self.get_state(element_state).is_open;
        
        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer);
        self.maybe_start_layer(renderer);
        {
            // Draw the dropdown selection.
            if let Some(pseudo_dropdown_selection) = self.pseudo_dropdown_selection.as_mut() {
                pseudo_dropdown_selection.internal.draw(renderer, font_system, taffy_tree, pseudo_dropdown_selection.internal.element_data().taffy_node_id.unwrap(), element_state, pointer, window.clone());
            }

            // Draw the dropdown list if it is open.
            if is_open && !self.children().is_empty() {
                self.pseudo_dropdown_list_element.draw(renderer, font_system, taffy_tree, self.pseudo_dropdown_list_element.element_data.taffy_node_id.unwrap(), element_state, pointer, window.clone());
                self.draw_children(renderer, font_system, taffy_tree, element_state, pointer, window.clone());
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
        self.merge_default_style();

        let state = self.get_state(element_state);
        let is_open = state.is_open;
        let mut child_nodes: Vec<NodeId> = Vec::new();

        // Find the `pseudo_dropdown_selection` element from the selected index.
        self.pseudo_dropdown_selection = if let Some(selected_index) = state.selected_item {
            if let Some(selected_element) = self.children_mut().get(selected_index) {
                Some(selected_element.clone())
            } else {
                self.children_mut().first().cloned()
            }
        } else {
            self.children_mut().first().cloned()
        };

        // Add the pseudo dropdown element to the Dropdown's layout tree.
        if let Some(selected_node) = self.pseudo_dropdown_selection.as_mut() {
            child_nodes.push(selected_node.internal.compute_layout(taffy_tree, element_state, scale_factor).unwrap());
        }

        // Compute the layout of the pseudo dropdown list if open.
        if is_open && !self.children().is_empty() {
            let dropdown_list_child_nodes: Vec<NodeId> = self.children_mut().iter_mut().filter_map(|child| { 
                child.internal.compute_layout(taffy_tree, element_state, scale_factor)
            }).collect();
            self.pseudo_dropdown_list_element.element_data.style = Style::merge(&Self::default_dropdown_list_style(), &self.pseudo_dropdown_list_element.element_data.style);
            
            let dropdown_list_node_id = taffy_tree.new_with_children(self.pseudo_dropdown_list_element.element_data.style.to_taffy_style_with_scale_factor(scale_factor), &dropdown_list_child_nodes).unwrap();

            // Add the pseudo dropdown list to the Dropdown's layout tree.
            child_nodes.push(dropdown_list_node_id);
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
        let state = self.get_state(element_state);
        let is_open = state.is_open;
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.finalize_borders();

        // Finalize the layout of the pseudo dropdown selection element.
        if let Some(dropdown_selection) = self.pseudo_dropdown_selection.as_mut() {
            let dropdown_selection_taffy = dropdown_selection.internal.element_data().taffy_node_id.unwrap();
            dropdown_selection.internal.finalize_layout(
                taffy_tree,
                dropdown_selection_taffy,
                self.element_data.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                font_system,
            );
        }

        // Finalize the layout of the pseudo dropdown list element when the list is open.
        if is_open && !self.children().is_empty() {
            let dropdown_list = taffy_tree.get_child_id(self.element_data.taffy_node_id.unwrap(), DROPDOWN_LIST_INDEX);
            self.pseudo_dropdown_list_element.element_data.taffy_node_id = Some(dropdown_list);
            self.pseudo_dropdown_list_element.finalize_layout(
                taffy_tree,
                dropdown_list,
                self.element_data.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                font_system,
            );

            for child in self.element_data.children.iter_mut() {
                let taffy_child_node_id = child.internal.element_data().taffy_node_id;
                if taffy_child_node_id.is_none() {
                    continue;
                }

                child.internal.finalize_layout(
                    taffy_tree,
                    taffy_child_node_id.unwrap(),
                    // The location of where the dropdown list starts for the list items.
                    self.pseudo_dropdown_list_element.element_data.computed_box.position,
                    z_index,
                    transform,
                    element_state,
                    pointer,
                    font_system,
                );
            }
        }
    }

    fn on_event(&self, message: &CraftMessage, element_state: &mut ElementStateStore, _font_system: &mut FontSystem) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<DropdownState>().unwrap();

        match message {
            CraftMessage::PointerButtonEvent(pointer_button) => {
                if !message.clicked() {
                    return UpdateResult::default();
                }

                for child in self.children().iter().enumerate() {
                    // Emit an event when a dropdown list item is selected and close the dropdown list.
                    // The emission of this event implies a DropdownToggled(false) event.
                    if child.1.in_bounds(pointer_button.position) {
                        // We need to retain the index of the selected item to render the `pseudo_dropdown_selection` element.
                        state.selected_item = Some(child.0);
                        state.is_open = false;

                        return UpdateResult::default().result_message(CraftMessage::DropdownItemSelected(state.selected_item.unwrap()))
                    }
                }

                // Emit an event when the dropdown list is opened or closed.
                let element_data = self.element_data();
                let transformed_border_rectangle = element_data.computed_box_transformed.border_rectangle();
                let dropdown_selection_in_bounds = transformed_border_rectangle.contains(&pointer_button.position);
                if dropdown_selection_in_bounds {
                    state.is_open = !state.is_open;
                    return UpdateResult::default().result_message(CraftMessage::DropdownToggled(state.is_open));
                }

            }
            CraftMessage::KeyboardInputEvent(_) => {}
            _ => {}
        }

        UpdateResult::default()
    }

    fn initialize_state(&self, _font_system: &mut FontSystem, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DropdownState::default()),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    /// The default style for the Dropdown container.
    fn default_style(&self) -> Style {
        let mut default_style = Style::default();

        *default_style.display_mut() = Display::Flex;
        *default_style.align_items_mut() = Some(AlignItems::Center);
        let vertical_padding = Unit::Px(8.0);
        let horizontal_padding = Unit::Px(12.0);
        *default_style.padding_mut() = [vertical_padding, horizontal_padding, vertical_padding, horizontal_padding];
        
        *default_style.min_width_mut() = Unit::Px(140.0);
        *default_style.min_height_mut() = Unit::Px(45.0);
        *default_style.background_mut() = Color::from_rgb8(240, 240, 240);
        
        let border_color = Color::from_rgb8(180, 180, 180);
        let border_radius = (6.0, 6.0);
        let border_width = Unit::Px(1.0);
        *default_style.border_radius_mut() = [border_radius, border_radius, border_radius, border_radius];
        *default_style.border_color_mut() = [border_color, border_color, border_color, border_color];
        *default_style.border_width_mut() = [border_width, border_width, border_width, border_width];
    
        default_style
    }
}

impl Dropdown {

    /// Sets the style of a dropdown list.
    pub fn dropdown_list_style(mut self, style: &Style) -> Self {
        self.pseudo_dropdown_list_element.element_data.style = *style;
        self
    }

    /// Returns the default style for a dropdown list.
    fn default_dropdown_list_style() -> Style {
        let mut default_style = Style::default();

        let vertical_padding = Unit::Px(8.0);
        let horizontal_padding = Unit::Px(12.0);
        *default_style.padding_mut() = [vertical_padding, horizontal_padding, vertical_padding, horizontal_padding];
        *default_style.min_width_mut() = Unit::Px(140.0);
        *default_style.min_height_mut() = Unit::Px(45.0);
        *default_style.background_mut() = Color::from_rgb8(220, 220, 220);


        let border_color = Color::from_rgb8(160, 160, 160);
        let border_radius = (6.0, 6.0);
        let border_width = Unit::Px(1.0);
        *default_style.border_radius_mut() = [border_radius, border_radius, border_radius, border_radius];
        *default_style.border_color_mut() = [border_color, border_color, border_color, border_color];
        *default_style.border_width_mut() = [border_width, border_width, border_width, border_width];

        *default_style.display_mut() = Display::Flex;
        *default_style.flex_direction_mut() = FlexDirection::Column;
        *default_style.position_mut() = Position::Absolute;
        // Position the dropdown list at the bottom of the dropdown.
        *default_style.inset_mut() = [Unit::Percentage(100.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0)];
        
        default_style
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a DropdownState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Dropdown {
        Dropdown {
            element_data: Default::default(),
            pseudo_dropdown_selection: Default::default(),
            pseudo_dropdown_list_element: Default::default(),
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Dropdown {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
