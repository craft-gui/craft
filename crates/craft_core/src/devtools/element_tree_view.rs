use crate::components::{ComponentId, ComponentSpecification};
use crate::devtools::dev_tools_colors::{
    CONTAINER_BACKGROUND_COLOR, ROW_BACKGROUND_COLOR, SELECTED_ROW_BACKGROUND_COLOR,
};
use crate::elements::element::Element;
use crate::elements::{Container, ElementStyles, Text};
use crate::style::{AlignItems, Display, FlexDirection};
use crate::Color;
use taffy::Overflow;

pub(crate) fn element_tree_view(
    root_element: &dyn Element,
    selected_element: Option<ComponentId>,
) -> ComponentSpecification {
    let mut element_tree = Container::new()
        .width("100%")
        .height("50%")
        .overflow(Overflow::Scroll)
        .max_height("50%")
        .padding("0px", "5px", "5px", "5px")
        .flex_direction(FlexDirection::Column);

    let mut elements: Vec<(&dyn Element, usize, bool)> = vec![(root_element, 0, true)];
    let mut element_count = 0;

    while let Some((element, indent, _is_last)) = elements.pop() {
        let row_color = if selected_element.is_some() && selected_element.unwrap() == element.component_id() {
            SELECTED_ROW_BACKGROUND_COLOR
        } else if element_count % 2 == 0 {
            ROW_BACKGROUND_COLOR
        } else {
            CONTAINER_BACKGROUND_COLOR
        };

        let id = element.component_id().to_string();

        let row_name = element.name().to_string();

        let mut row = Container::new()
            .push(
                Text::new(row_name.as_str())
                    .padding("0px", "0px", "0px", format!("{}px", indent * 10).as_str())
                    .color(Color::WHITE)
                    .id(id.as_str())
                    .component(),
            )
            .display(Display::Flex)
            .align_items(AlignItems::Center)
            .background(row_color)
            .padding("6px", "6px", "6px", "6px")
            .height("40px")
            .max_height("40px")
            .key(element_count.to_string().as_str())
            .id(id.as_str())
            .width("100%");

        if let Some(custom_id) = element.get_id() {
            let user_id_color = Color::from_rgb8(68, 147, 248);
            row = row.push(
                Container::new()
                    .push(
                        Text::new(custom_id.as_str())
                            .color(Color::WHITE)
                            .margin("2.5px", "10px", "2.5px", "10px")
                            .id(id.as_str()),
                    )
                    .id(id.as_str())
                    .border_width("2px", "2px", "2px", "2px")
                    .border_color(user_id_color)
                    .border_radius(100.0, 100.0, 100.0, 100.0)
                    .margin("0px", "0px", "0px", "5px")
                    .component(),
            );
        }


        let user_id_color = Color::from_rgb8(68, 147, 248);
        row = row.push(
            Container::new()
                .push(
                    Text::new(element.component_id().to_string().as_str())
                        .color(Color::WHITE)
                        .margin("2.5px", "10px", "2.5px", "10px")
                        .id(id.as_str()),
                )
                .id(id.as_str())
                .border_width("2px", "2px", "2px", "2px")
                .border_color(user_id_color)
                .border_radius(100.0, 100.0, 100.0, 100.0)
                .margin("0px", "0px", "0px", "5px")
                .component(),
        );

        element_tree = element_tree.push(row);

        let children = element.children();
        for (i, child) in children.iter().enumerate().rev() {
            let is_last = i == children.len() - 1;
            elements.push((*child, indent + 1, is_last));
        }

        element_count += 1;
    }

    element_tree.component()
}
