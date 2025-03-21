#[path = "../util.rs"]
mod util;

use util::setup_logging;

use oku::components::ComponentSpecification;
use oku::components::{Component, UpdateResult};
use oku::elements::ElementStyles;
use oku::elements::TextInput;
use oku::elements::{Container, Text};
use oku::events::Event;
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku::RendererType;
use oku::elements::{Dropdown, Switch};
use oku_core::style::Display;

#[derive(Default, Copy, Clone)]
pub struct InputsExample {
}

impl InputsExample {
    const DROPDOWN_ITEMS: [&'static str; 4] = ["Dropdown Item 1", "Dropdown Item 2", "Dropdown Item 3", "Dropdown Item 4"];
}

impl Component for InputsExample {
    type Props = ();

    fn view_with_no_global_state(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .padding("20px", "20px", "20px", "20px")
            .display(Display::Block)
            .push(Text::new("Common Input Elements:").font_size(24.0))
            .push(TextInput::new("Hi").id("text_input").margin("10px", "0px", "0px", "0px"))
            .push(Switch::new().default_toggled(true).margin("10px", "0px", "0px", "0px"))
            .push(Dropdown::new()
                .push(Text::new(Self::DROPDOWN_ITEMS[0]))
                .push(Text::new(Self::DROPDOWN_ITEMS[1]))
                .push(Text::new(Self::DROPDOWN_ITEMS[2]))
                .push(Text::new(Self::DROPDOWN_ITEMS[3]))
                .margin("10px", "0px", "0px", "0px")
            )
            .component()
    }

    fn update_with_no_global_state(_state: &mut Self, _props: &Self::Props, _event: Event) -> UpdateResult {
        UpdateResult::new()
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();

    oku_main_with_options(
        InputsExample::component(),
        Box::new(()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "inputs".to_string(),
        }),
    );
}
