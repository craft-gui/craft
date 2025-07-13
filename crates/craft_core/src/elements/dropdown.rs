use crate::components::component::ComponentSpecification;
use crate::components::Event;
use crate::components::Props;
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::elements::{Container, StatefulElement};
use crate::events::CraftMessage;
use crate::generate_component_methods;
use craft_primitives::geometry::{Point, Rectangle, TrblRectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use craft_renderer::renderer::RenderList;
use crate::style::{AlignItems, Display, FlexDirection, Style, Unit};
use crate::text::text_context::TextContext;
use peniko::Color;
use std::any::Any;
use std::sync::Arc;
use kurbo::Affine;
use taffy::{NodeId, Position, TaffyTree, TraversePartialTree};
use winit::window::Window;
use smol_str::SmolStr;

/// The index of the dropdown list in the layout tree.
const DROPDOWN_LIST_INDEX: usize = 1;

/// An element for displaying a list of items in a dropdown. By default, the first list item will be shown, otherwise show the selected item.
#[derive(Clone, Default)]
pub struct Dropdown {
    pub element_data: ElementData,
    /// A copy of the currently selected element, this is not stored in the user tree nor will it receive events.
    /// This is copied, so that we can change the location and render it in the dropdown container.
    pseudo_dropdown_selection: Option<ElementBoxed>,
    /// An element not in the user tree. Created, so that we can utilize our existing functionality (like scrollbars).
    pseudo_dropdown_list_element: Container,
    
    default_item: usize,
}

#[derive(Clone, Copy, Default)]
pub struct DropdownState {
    /// Whether the dropdown list is visible or not.
    is_open: bool,
    /// The index of the currently selected item in the dropdown list.
    /// For example if you select the first item, `selected_item` will be 0.
    selected_item: Option<usize>,
}

impl StatefulElement<DropdownState> for Dropdown {}

impl Element for Dropdown {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    /// Checks if a point is in the dropdown selection and/or list.
    fn in_bounds(&self, point: Point) -> bool {
        // Check the bounds of the dropdown selection.
        let element_data = self.element_data();
        let transformed_border_rectangle = element_data.layout_item.computed_box_transformed.border_rectangle();
        let dropdown_selection_in_bounds = transformed_border_rectangle.contains(&point);

        // Check the bounds of the dropdown list items.
        let mut dropdown_list_in_bounds = false;
        for child in self.children() {
            if child.internal.in_bounds(point) {
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

        let state = self.state(element_state);
        let is_open = state.is_open;

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, element_state, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        {
            // Draw the dropdown selection.
            if let Some(pseudo_dropdown_selection) = self.pseudo_dropdown_selection.as_mut() {
                pseudo_dropdown_selection.internal.draw(renderer, text_context, element_state, pointer, window.clone(), scale_factor);
            }

            // CLEANUP: We could make pseudo_dropdown_list_element an Overlay, but below we draw the overlay then the children.
            //          We didn't do that for any particular reason, mainly just because we need to add more abstractions around drawing.
            renderer.start_overlay();
            // Draw the dropdown list if it is open.
            if is_open && !self.children().is_empty() {
                self.pseudo_dropdown_list_element.draw(renderer, text_context, element_state, pointer, window.clone(), scale_factor);
                self.draw_children(renderer, text_context, element_state, pointer, window.clone(), scale_factor);
            }
            renderer.end_overlay();
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

        let state = self.state(element_state);
        let is_open = state.is_open;
        let default_item = self.default_item;

        // Find the `pseudo_dropdown_selection` element from the selected index.
        self.pseudo_dropdown_selection = if let Some(selected_index) = state.selected_item {
            if let Some(selected_element) = self.children_mut().get(selected_index) {
                Some(selected_element.clone())
            } else {
                self.children_mut().first().cloned()
            }
        } else {
            self.children_mut().get(default_item).cloned()
        };

        // Add the pseudo dropdown element to the Dropdown's layout tree.
        if let Some(selected_node) = self.pseudo_dropdown_selection.as_mut() {
            self.element_data.layout_item.push_child(&selected_node.internal.compute_layout(
                taffy_tree,
                element_state,
                scale_factor,
            ));
        }

        // Compute the layout of the pseudo dropdown list if open.
        if is_open && !self.children().is_empty() {
            let dropdown_list_child_nodes: Vec<NodeId> = self
                .children_mut()
                .iter_mut()
                .filter_map(|child| child.internal.compute_layout(taffy_tree, element_state, scale_factor))
                .collect();
            self.pseudo_dropdown_list_element.element_data.style = Style::merge(
                &Self::default_dropdown_list_style(),
                &self.pseudo_dropdown_list_element.element_data.style,
            );

            let dropdown_list_node_id = taffy_tree
                .new_with_children(
                    self.pseudo_dropdown_list_element.element_data.style.to_taffy_style(),
                    &dropdown_list_child_nodes,
                )
                .unwrap();

            // Add the pseudo dropdown list to the Dropdown's layout tree.
            self.element_data.layout_item.push_child(&Some(dropdown_list_node_id));
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
        transform: Affine,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let state = self.state(element_state);
        let is_open = state.is_open;
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);
        self.finalize_borders(element_state);

        // Finalize the layout of the pseudo dropdown selection element.
        if let Some(dropdown_selection) = self.pseudo_dropdown_selection.as_mut() {
            let dropdown_selection_taffy =
                dropdown_selection.internal.element_data().layout_item.taffy_node_id.unwrap();
            dropdown_selection.internal.finalize_layout(
                taffy_tree,
                dropdown_selection_taffy,
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                text_context,
                None,
            );
        }

        // Finalize the layout of the pseudo dropdown list element when the list is open.
        if is_open && !self.children().is_empty() {
            let dropdown_list =
                taffy_tree.get_child_id(self.element_data.layout_item.taffy_node_id.unwrap(), DROPDOWN_LIST_INDEX);
            self.pseudo_dropdown_list_element.element_data.layout_item.taffy_node_id = Some(dropdown_list);
            self.pseudo_dropdown_list_element.finalize_layout(
                taffy_tree,
                dropdown_list,
                self.computed_box().position,
                z_index,
                transform,
                element_state,
                pointer,
                text_context,
                None,
            );

            for child in self.element_data.children.iter_mut() {
                let taffy_child_node_id = child.internal.element_data().layout_item.taffy_node_id;
                if taffy_child_node_id.is_none() {
                    continue;
                }

                child.internal.finalize_layout(
                    taffy_tree,
                    taffy_child_node_id.unwrap(),
                    // The location of where the dropdown list starts for the list items.
                    self.pseudo_dropdown_list_element.element_data.layout_item.computed_box.position,
                    z_index,
                    transform,
                    element_state,
                    pointer,
                    text_context,
                    None,
                );
            }
        }
    }

    fn resolve_clip(&mut self, _clip_bounds: Option<Rectangle>) {
        self.element_data.layout_item.clip_bounds = None;
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
        event: &mut Event,
        target: Option<&dyn Element>,
        _current_target: Option<&dyn Element>,
    ) {
        event.propagate = true;
        event.prevent_defaults = true;

        self.on_style_event(message, element_state, should_style, event);
        self.maybe_unset_focus(message, event, target);
        let (state, _base_state) = self.state_and_base_mut(element_state);

        match message {
            CraftMessage::PointerButtonUp(pointer_button) => {
                if !message.clicked() {
                    return;
                }

                for child in self.children().iter().enumerate() {
                    // Emit an event when a dropdown list item is selected and close the dropdown list.
                    // The emission of this event implies a DropdownToggled(false) event.
                    if child.1.internal.in_bounds(pointer_button.state.position) {
                        // We need to retain the index of the selected item to render the `pseudo_dropdown_selection` element.
                        state.selected_item = Some(child.0);
                        state.is_open = false;

                        event.result_message(CraftMessage::DropdownItemSelected(state.selected_item.unwrap()));
                        event.prevent_defaults();
                        event.prevent_propagate();
                        return;
                    }
                }

                // Emit an event when the dropdown list is opened or closed.
                let element_data = self.element_data();
                let transformed_border_rectangle = element_data.layout_item.computed_box_transformed.border_rectangle();
                let dropdown_selection_in_bounds =
                    transformed_border_rectangle.contains(&pointer_button.state.position);
                if dropdown_selection_in_bounds {
                    state.is_open = !state.is_open;
                    event.result_message(CraftMessage::DropdownToggled(state.is_open));
                }
            }
            CraftMessage::KeyboardInputEvent(_) => {}
            _ => {}
        }
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DropdownState::default()),
        }
    }

    /// The default style for the Dropdown container.
    fn default_style(&self) -> Style {
        let mut default_style = Style::default();

        default_style.set_display(Display::Flex);
        default_style.set_align_items(Some(AlignItems::Center));

        let vertical_padding = Unit::Px(8.0);
        let horizontal_padding = Unit::Px(12.0);
        default_style.set_padding(TrblRectangle::new(
            vertical_padding,
            horizontal_padding,
            vertical_padding,
            horizontal_padding,
        ));

        default_style.set_min_width(Unit::Px(140.0));
        default_style.set_min_height(Unit::Px(45.0));
        default_style.set_background(Color::from_rgb8(240, 240, 240));

        let border_color = Color::from_rgb8(180, 180, 180);
        let border_radius = (6.0, 6.0);
        let border_width = Unit::Px(1.0);

        default_style.set_border_radius([
            border_radius,
            border_radius,
            border_radius,
            border_radius,
        ]);

        default_style.set_border_color(TrblRectangle::new_all(border_color));
        default_style.set_border_width(TrblRectangle::new_all(border_width));
        
        default_style
    }
}

impl Dropdown {
    /// Sets the style of a dropdown list.
    pub fn dropdown_list_style(mut self, style: Style) -> Self {
        self.pseudo_dropdown_list_element.element_data.style = style;
        self
    }

    /// Returns the default style for a dropdown list.
    fn default_dropdown_list_style() -> Style {
        let mut default_style = Style::default();

        let vertical_padding = Unit::Px(8.0);
        let horizontal_padding = Unit::Px(12.0);
        default_style.set_padding(TrblRectangle::new(
            vertical_padding,
            horizontal_padding,
            vertical_padding,
            horizontal_padding,
        ));

        default_style.set_min_width(Unit::Px(140.0));
        default_style.set_min_height(Unit::Px(45.0));
        default_style.set_background(Color::from_rgb8(220, 220, 220));

        let border_color = Color::from_rgb8(160, 160, 160);
        let border_radius = (6.0, 6.0);
        let border_width = Unit::Px(1.0);

        default_style.set_border_radius([
            border_radius,
            border_radius,
            border_radius,
            border_radius,
        ]);

        default_style.set_border_color(TrblRectangle::new_all(border_color));
        default_style.set_border_width(TrblRectangle::new_all(border_width));

        default_style.set_display(Display::Flex);
        default_style.set_flex_direction(FlexDirection::Column);
        default_style.set_position(Position::Absolute);

        let mut inset = default_style.inset();
        inset.top = Unit::Percentage(100.0);
        default_style.set_inset(inset);

        default_style
    }

    pub fn new() -> Dropdown {
        Dropdown {
            element_data: Default::default(),
            pseudo_dropdown_selection: Default::default(),
            pseudo_dropdown_list_element: Default::default(),
            default_item: 0,
        }
    }
    
    pub fn set_default(mut self, index: usize) -> Self {
        self.default_item = index;
        self
    }

    generate_component_methods!();
}

impl ElementStyles for Dropdown {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
