use util::{setup_logging, ExampleProps};

use craft::components::ComponentSpecification;
use craft::components::Component;
use craft::components::Context;
use craft::craft_main;
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::elements::{Dropdown, Slider, SliderDirection, Switch, TextInput, TinyVg};
use craft::events::CraftMessage::DropdownItemSelected;
use craft::events::CraftMessage::{SliderValueChanged, SwitchToggled, TextInputChanged};
use craft::events::Message::CraftMessage;
use craft::ResourceIdentifier;
use craft::style::{AlignItems, Weight};
use craft::style::{Display, FlexDirection, Overflow, Wrap};
use craft::{Color, CraftOptions};

#[derive(Clone)]
pub struct Tour {
    text_input_value: String,
    slider_value: f64,
    switch_value: bool,
    dropdown_value: Option<usize>,
}

impl Default for Tour {
    fn default() -> Self {
        Self {
            text_input_value: "".to_string(),
            slider_value: 0.0,
            switch_value: DEFAULT_SWITCH_VALUE,
            dropdown_value: None,
        }
    }
}

impl Tour {
    const DROPDOWN_ITEMS: [&'static str; 4] =
        ["Dropdown Item 1", "Dropdown Item 2", "Dropdown Item 3", "Dropdown Item 4"];
}

const DEFAULT_SWITCH_VALUE: bool = true;

impl Component for Tour {
    type Props = ExampleProps;
    type Message = ();
    type GlobalState = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let section = |title: &str, content: ComponentSpecification| {
            Container::new()
                .padding("16px", "16px", "16px", "16px")
                .margin("0px", "0px", "20px", "0px")
                .border_width("1px", "1px", "1px", "1px")
                .border_radius(8.0, 8.0, 8.0, 8.0)
                .background(Color::WHITE)
                .border_color(Color::from_rgb8(220, 220, 222))
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(
                    Text::new(title).font_size(20.0).font_weight(Weight::SEMIBOLD).margin("0px", "0px", "12px", "0px"),
                )
                .push(content)
                .component()
        };

        let labeled_row = |label: &str, element: ComponentSpecification| {
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap("12px")
                .margin("0px", "0px", "12px", "0px")
                .push(Text::new(label).font_size(16.0))
                .push(element)
                .component()
        };

        let secondary_text_color = Color::from_rgb8(85, 85, 85);

        let text_section = section(
            "Text Input",
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(labeled_row(
                    "Input:",
                    TextInput::new(context.state().text_input_value.as_str())
                        .min_width("200px")
                        .component(),
                ))
                .push(
                    Text::new(format!("Preview: {}", context.state().text_input_value).as_str())
                        .font_size(14.0)
                        .color(secondary_text_color),
                )
                .component(),
        );

        let switch_section = section(
            "Switch",
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(labeled_row(
                    "Enabled:",
                    Switch::new(24.0).default_toggled(DEFAULT_SWITCH_VALUE).spacing(4.0).round().component(),
                ))
                .push(
                    Text::new(if context.state().switch_value { "On" } else { "Off" }).font_size(14.0).color(secondary_text_color),
                )
                .component(),
        );

        let slider_section = section(
            "Slider",
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(labeled_row(
                    "Level:",
                    Slider::new(16.0).direction(SliderDirection::Horizontal).step(1.0).round().component(),
                ))
                .push(Text::new(format!("{}", context.state().slider_value).as_str()).font_size(14.0).color(secondary_text_color))
                .component(),
        );

        let bottom_section = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .wrap(Wrap::Wrap)
            .gap("20px")
            .push(
                section(
                    "Dropdown",
                    Dropdown::new()
                        .push(Text::new(Self::DROPDOWN_ITEMS[0]).disable_selection())
                        .push(Text::new(Self::DROPDOWN_ITEMS[1]).disable_selection())
                        .push(Text::new(Self::DROPDOWN_ITEMS[2]).disable_selection())
                        .push(Text::new(Self::DROPDOWN_ITEMS[3]).disable_selection())
                        .min_width("200px")
                        .component(),
                )
                .push(
                    Text::new(
                        format!(
                            "Selected: {}",
                            context.state().dropdown_value.map(|index| Self::DROPDOWN_ITEMS[index]).unwrap_or("None")
                        )
                        .as_str(),
                    )
                    .font_size(14.0)
                    .margin("12px", "0px", "0px", "0px")
                    .color(secondary_text_color),
                ),
            )
            .push(section(
                "TinyVG",
                TinyVg::new(ResourceIdentifier::Bytes(include_bytes!("tiger.tvg"))).max_width("200px").component(),
            ))
            .component();

        Container::new()
            .overflow_y(if context.props().show_scrollbar { Overflow::Scroll } else { Overflow::Visible })
            .padding("24px", "24px", "24px", "24px")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .max_height("100%")
            .gap("16px")
            .push(Text::new("Tour").font_size(28.0).font_weight(Weight::BOLD).margin("0px", "0px", "16px", "0px"))
            .push(text_section)
            .push(switch_section)
            .push(slider_section)
            .push(bottom_section)
            .component()
    }

    fn update(context: &mut Context<Self>) {
        if let CraftMessage(TextInputChanged(str)) = context.message() {
            context.state_mut().text_input_value = str.clone();
            context.event_mut().prevent_defaults();
            context.event_mut().prevent_propagate();
            return;
        }

        if let CraftMessage(SliderValueChanged(val)) = context.message() {
            context.state_mut().slider_value = *val;
            context.event_mut().prevent_defaults();
            context.event_mut().prevent_propagate();
            return;
        }

        if let CraftMessage(SwitchToggled(val)) = context.message() {
            context.state_mut().switch_value = *val;
            context.event_mut().prevent_defaults();
            context.event_mut().prevent_propagate();
            return;
        }

        if let CraftMessage(DropdownItemSelected(item)) = context.message() {
            context.state_mut().dropdown_value = Some(*item);
            context.event_mut().prevent_defaults();
            context.event_mut().prevent_propagate();
        }
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();
    craft_main(Tour::component(), (), CraftOptions::basic("Tour"));
}
