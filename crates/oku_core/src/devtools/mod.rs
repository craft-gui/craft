use std::fmt::format;
use taffy::Overflow;
use winit::event::{ElementState, MouseButton};
use crate::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use crate::components::props::Props;
use crate::elements::{Container, ElementStyles, Text};
use crate::elements::element::{Element, ElementBox};
use crate::events::{Event, Message, OkuMessage};
use crate::geometry::Size;
use crate::renderer::color::Color;
use crate::style::{FlexDirection, Unit};
use crate::elements;
use crate::style::Display::Flex;

pub(crate) struct DevTools {
    pub width: Unit,
    pub height: Unit,

    pub selected_element: Option<ComponentId>,
}

impl Default for DevTools {
    fn default() -> Self {
        Self {
            width: Unit::Percentage(100.0),
            height: Unit::Percentage(100.0),
            selected_element: None,
        }
    }
}

impl Component for DevTools {
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
            
            element_tree = element_tree.push(
                Container::new()
                    .push(
                        Text::new(format!("{}", element.name()).as_str())
                            .padding("0px", "0px", "0px", format!("{}px", indent * 10).as_str())
                            .color(Color::WHITE)
                            .id(id.as_str())
                            .component()
                    )
                    .background(background_color)
                    .padding("5px", "5px", "5px", "5px")
                    .key(element_count.to_string().as_str())
                    .id(id.as_str())
                    .width("100%")
            );

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

        Container::new()
            .display(Flex)
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
        }

        UpdateResult::default()
    }
}

pub fn dev_tools_view(root: &Box<dyn Element>) -> ComponentSpecification {
    DevTools::component().props(Props::new(Some(root.clone())))
}