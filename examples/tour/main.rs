use util::setup_logging;

use craft::components::ComponentId;
use craft::components::ComponentSpecification;
use craft::components::{Component, Event};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::elements::{Dropdown, Slider, SliderDirection, Switch, TextInput, TinyVg};
use craft::events::CraftMessage::DropdownItemSelected;
use craft::events::CraftMessage::{SliderValueChanged, SwitchToggled, TextInputChanged};
use craft::events::Message::CraftMessage;
use craft::events::Message;
use craft::resource_manager::ResourceIdentifier;
use craft::style::{AlignItems, Weight};
use craft::style::{Display, FlexDirection, Overflow, Wrap};
use craft::RendererType;
use craft::{craft_main_with_options, WindowContext};
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
    type Props = ();
    type Message = ();
    type GlobalState = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext
    ) -> ComponentSpecification {

        let section = |title: &str, content: ComponentSpecification| {
            Container::new()
                .padding("16px", "16px", "16px", "16px")
                .margin("0px", "0px", "20px", "0px")
                .border_radius(8.0, 8.0, 8.0, 8.0)
                .background(Color::WHITE)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(Text::new(title).font_size(20.0).font_weight(Weight::SEMIBOLD).margin("0px", "0px", "12px", "0px"))
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
                .push(
                    labeled_row("Input:", TextInput::new(self.text_input_value.as_str()).id("text_input").min_width("200px").component())
                )
                .push(Text::new(format!("Preview: {}", self.text_input_value).as_str())
                    .font_size(14.0)
                    .color(secondary_text_color)
                )
                .component()
        );

        let switch_section = section(
            "Switch",
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(
                    labeled_row(
                        "Enabled:",
                        Switch::new(24.0)
                            .default_toggled(DEFAULT_SWITCH_VALUE)
                            .spacing(4.0)
                            .round()
                            .component()
                    )
                )
                .push(Text::new(if self.switch_value { "On" } else { "Off" }).font_size(14.0).color(secondary_text_color))
                .component()
        );

        let slider_section = section(
            "Slider",
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(
                    labeled_row(
                        "Level:",
                        Slider::new(16.0)
                            .direction(SliderDirection::Horizontal)
                            .step(1.0)
                            .round()
                            .component()
                    )
                )
                .push(Text::new(format!("{}", self.slider_value).as_str()).font_size(14.0).color(secondary_text_color))
                .component()
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
                        .push(Text::new(Self::DROPDOWN_ITEMS[0]))
                        .push(Text::new(Self::DROPDOWN_ITEMS[1]))
                        .push(Text::new(Self::DROPDOWN_ITEMS[2]))
                        .push(Text::new(Self::DROPDOWN_ITEMS[3]))
                        .min_width("200px")
                        .component()
                )
                .push(
                    Text::new(
                        format!("Selected: {}", self.dropdown_value.map(|index| Self::DROPDOWN_ITEMS[index]).unwrap_or("None")).as_str()
                    )
                    .font_size(14.0)
                    .margin("12px", "0px", "0px", "0px")
                    .color(secondary_text_color)
                )
            )
            .push(
                section(
                    "TinyVG",
                    TinyVg::new(ResourceIdentifier::Bytes(include_bytes!("tiger.tvg")))
                        .max_width("200px")
                        .component()
                )
            )
            .component();

        Container::new()
            .overflow_y(Overflow::Scroll)
            .padding("24px", "24px", "24px", "24px")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .max_height("100%")
            .background(Color::from_rgb8(220, 220, 222))
            .gap("16px")
            .push(Text::new("Tour").font_size(28.0).font_weight(Weight::BOLD).margin("0px", "0px", "16px", "0px"))
            .push(text_section)
            .push(switch_section)
            .push(slider_section)
            .push(bottom_section)
            .component()
    }

    fn update(&mut self, _global_state: &mut Self::GlobalState, _props: &Self::Props, event: &mut Event, message: &Message) {
        if let CraftMessage(TextInputChanged(str)) = message {
            self.text_input_value = str.clone();
            event.prevent_defaults();
            event.prevent_propagate();
            return;
        }

        if let CraftMessage(SliderValueChanged(val)) = message {
            self.slider_value = *val;
            event.prevent_defaults();
            event.prevent_propagate();
            return;
        }

        if let CraftMessage(SwitchToggled(val)) = message {
            self.switch_value = *val;
            event.prevent_defaults();
            event.prevent_propagate();
            return;
        }

        if let CraftMessage(DropdownItemSelected(item)) = message {
            self.dropdown_value = Some(*item);
            event.prevent_defaults();
            event.prevent_propagate();
        }
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();

    craft_main_with_options(
        Tour::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Tour".to_string(),
            ..Default::default()
        }),
    );
}
