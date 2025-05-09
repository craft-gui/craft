use util::setup_logging;

use craft::components::ComponentId;
use craft::components::ComponentSpecification;
use craft::components::{Component, UpdateResult};
use craft::elements::{Container, Text};
use craft::elements::ElementStyles;
use craft::events::CraftMessage::{SliderValueChanged, SwitchToggled, TextInputChanged};
use craft::events::Event;
use craft::events::Message::CraftMessage;
use craft::style::{Display, FlexDirection, Overflow};
use craft::CraftOptions;
use craft::RendererType;
use craft::{craft_main_with_options, WindowContext};
use craft::elements::{Dropdown, Slider, SliderDirection, Switch, TextInput, TinyVg};
use craft::resource_manager::ResourceIdentifier;

#[derive(Clone)]
pub struct Tour {
    my_text: String,
    slider_value: f64,
    switch_value: bool,
}

impl Default for Tour {
    fn default() -> Self {
        Self {
            my_text: "".to_string(),
            slider_value: 0.0,
            switch_value: DEFAULT_SWITCH_VALUE,
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

    fn view_with_no_global_state(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window_context: &WindowContext
    ) -> ComponentSpecification {
        
        let container = Container::new()
            .overflow_y(Overflow::Scroll)
            .padding("20px", "20px", "20px", "20px")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .max_height("100%")
            .gap("10px")
            .push(Text::new("Tour:").font_size(24.0))
            .push(TextInput::new(state.my_text.as_str()).id("text_input"))
            .push(Text::new(format!("Value: {}", state.my_text).as_str()).margin("0px", "0px", "25px", "0px"))
            .push(Switch::new(24.0).spacing(4.0).round().default_toggled(DEFAULT_SWITCH_VALUE))
            .push(Text::new(format!("Value: {}", if state.switch_value { "On" } else { "Off" }).as_str()).margin("0px", "0px", "25px", "0px"))
            .push(Slider::new(16.0).direction(SliderDirection::Horizontal).step(1.0).round())
            .push(Text::new(format!("Value: {:?}", state.slider_value).as_str()).margin("0px", "0px", "25px", "0px"))
            .push(
                 Dropdown::new()
                     .push(Text::new(Self::DROPDOWN_ITEMS[0]))
                     .push(Text::new(Self::DROPDOWN_ITEMS[1]))
                     .push(Text::new(Self::DROPDOWN_ITEMS[2]))
                     .push(Text::new(Self::DROPDOWN_ITEMS[3])),
            )
            .push(
                TinyVg::new(ResourceIdentifier::Bytes(include_bytes!("tiger.tvg"))).max_width("40%")
            )
            .component();
        
        container
    }

    fn update_with_no_global_state(state: &mut Self, _props: &Self::Props, event: Event, _window_context: &mut WindowContext) -> UpdateResult {
        if let CraftMessage(TextInputChanged(str)) = event.message {
            state.my_text = str.clone();
            return UpdateResult::new().prevent_defaults().prevent_propagate();
        }

        if let CraftMessage(SliderValueChanged(val)) = event.message {
            state.slider_value = *val;
            return UpdateResult::new().prevent_defaults().prevent_propagate();
        }

        if let CraftMessage(SwitchToggled(val)) = event.message {
            state.switch_value = *val;
            return UpdateResult::new().prevent_defaults().prevent_propagate();
        }

        UpdateResult::new()
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
