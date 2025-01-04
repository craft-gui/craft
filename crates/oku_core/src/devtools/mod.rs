mod dev_tools_widget;

use crate::components::props::Props;
use crate::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use crate::devtools::dev_tools_widget::DevTools;
use crate::elements::element::Element;
use crate::elements::{Container, ElementStyles, Text};
use crate::events::{Event, Message, OkuMessage};
use crate::renderer::color::Color;
use crate::style::Display::Flex;
use crate::style::{AlignItems, Display, FlexDirection, Unit};
use taffy::Overflow;
use winit::event::{ElementState, MouseButton};

pub(crate) struct DevToolsComponent {
    pub width: Unit,
    pub height: Unit,

    pub selected_element: Option<ComponentId>,
    pub inspector_hovered_element: Option<ComponentId>,
}

impl Default for DevToolsComponent {
    fn default() -> Self {
        Self {
            width: Unit::Percentage(100.0),
            height: Unit::Percentage(100.0),
            selected_element: None,
            inspector_hovered_element: None,
        }
    }
}

impl Component for DevToolsComponent {
    type Props = Option<Box<dyn Element>>;

    fn view(state: &Self, props: &Self::Props, children: Vec<ComponentSpecification>) -> ComponentSpecification {

        let root = props.as_ref().unwrap().clone();
        let mut element_tree = Container::new()
            .width("100%")
            .height("50%")
            .overflow(Overflow::Scroll)
            .max_height("50%")
            .padding("5px", "5px", "5px", "5px")
            .flex_direction(FlexDirection::Column);

        let mut elements: Vec<(&dyn Element, usize, bool)> = vec![(root.as_ref(), 0, true)];
        let mut element_count = 0;

        while let Some((element, indent, is_last)) = elements.pop() {


            let background_color = if state.selected_element.is_some() && state.selected_element.unwrap() == element.component_id() {
                Color::rgba(45, 45, 90, 255)
            } else if element_count % 2 == 0 {
                Color::rgba(60, 60, 60, 255)
            } else {
                Color::rgba(45, 45, 45, 255)
            };
            
            let id = element.component_id().to_string();

            let mut row_name = element.name().to_string();

            let mut row =  Container::new()
                .push(
                    Text::new(row_name.as_str())
                        .padding("0px", "0px", "0px", format!("{}px", indent * 10).as_str())
                        .color(Color::WHITE)
                        .id(id.as_str())
                        .component()
                )
                .display(Display::Flex)
                .align_items(AlignItems::Center)
                .background(background_color)
                .padding("6px", "6px", "6px", "6px")
                .height("40px")
                .max_height("40px")
                .key(element_count.to_string().as_str())
                .id(id.as_str())
                .width("100%");

            if let Some(custom_id) = element.get_id() {
                let user_id_color = Color::rgba(68, 147, 248, 255);
                row = row.push(Container::new()
                    .push(Text::new(custom_id.as_str()).color(Color::WHITE).margin("2.5px", "10px", "2.5px", "10px").id(id.as_str()))
                    .id(id.as_str())
                    .border_width("2px", "2px", "2px", "2px")
                    .border_color(user_id_color)
                    .border_radius(100.0, 100.0, 100.0, 100.0)
                    .margin("0px", "0px", "0px", "5px")
                    .component()
                );
            }

            element_tree = element_tree.push(row);

            let children = element.children();
            for (i, child) in children.iter().enumerate().rev() {
                let is_last = i == children.len() - 1;
                elements.push((*child, indent + 1, is_last));
            }

            element_count += 1;
        }

        let mut styles_window = Container::new()
            .width(state.width)
            .display(Flex)
            .flex_direction(FlexDirection::Column)
            .margin("10px", "10px", "10px", "10px")
            .height("50%")
            .max_height("50%")
            .overflow(Overflow::Scroll)
            .push(Container::new()
                .border_width("0px", "0px", "2px", "0px").border_color(Color::WHITE)
                .push(Text::new("Styles Window").color(Color::rgba(230, 230, 230, 255)).margin("10px", "0px", "0px", "0px"))
            )
            .component();
        
        let mut selected_element: Option<Box<&dyn Element>> = None;
        if state.selected_element.is_some() {
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != state.selected_element.unwrap() {
                    continue;
                }

                selected_element = Some(Box::new(<&dyn Element>::clone(element)));
                break;
            }

            if let Some(selected_element) = selected_element {
                styles_window = styles_window.push(Text::new(format!("Margin Top: {}", selected_element.style().margin[0].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Margin Right: {}", selected_element.style().margin[1].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Margin Bottom: {}", selected_element.style().margin[2].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Margin Left: {}", selected_element.style().margin[3].to_string()).as_str()).color(Color::WHITE));

                styles_window = styles_window.push(Text::new(format!("Padding Top: {}", selected_element.style().padding[0].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Padding Right: {}", selected_element.style().padding[1].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Padding Bottom: {}", selected_element.style().padding[2].to_string()).as_str()).color(Color::WHITE));
                styles_window = styles_window.push(Text::new(format!("Padding Left: {}", selected_element.style().padding[3].to_string()).as_str()).color(Color::WHITE));
            }
        }

        DevTools::new()
            .display(Flex)
            .push_inspector_root_element(&root)
            .push_element_to_inspect(state.selected_element)
            .push_inspector_hovered_element(state.inspector_hovered_element)
            .flex_direction(FlexDirection::Column)
            .background(Color::rgba(45, 45, 45, 255))
            .width(state.width)
            .height(state.height)
            .max_height(state.height)
            .push(element_tree)
            .push(styles_window)
            .component()
    }

    fn update(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        if let Some(id) = event.target {

            if let Message::OkuMessage(OkuMessage::PointerButtonEvent(pointer_button)) = event.message {
                if pointer_button.button.mouse_button() == MouseButton::Left
                    && pointer_button.state == ElementState::Pressed
                {
                    let component_id: ComponentId = id.parse().unwrap();
                    state.selected_element = Some(component_id);
                }
            }
            
            if let Message::OkuMessage(OkuMessage::PointerMovedEvent(pointer_moved_event)) = event.message {
                let component_id: ComponentId = id.parse().unwrap();
                state.inspector_hovered_element = Some(component_id);
            }
        } else {
            state.inspector_hovered_element = None;
        }

        UpdateResult::default()
    }
}

pub fn dev_tools_view(root: &Box<dyn Element>) -> ComponentSpecification {
    DevToolsComponent::component().props(Props::new(Some(root.clone())))
}