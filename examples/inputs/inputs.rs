#[path = "../util.rs"]
mod util;

use util::setup_logging;

use craft::components::ComponentSpecification;
use craft::components::{Component, UpdateResult};
use craft::craft_main_with_options;
use craft::elements::ElementStyles;
use craft::elements::TextInput;
use craft::elements::{Container, Text};
use craft::elements::{Dropdown, Switch};
use craft::events::Event;
use craft::CraftOptions;
use craft::RendererType;
use craft_core::components::ComponentId;
use craft_core::events::CraftMessage::TextInputChanged;
use craft_core::events::Message::CraftMessage;
use craft_core::style::Display;

#[derive(Default, Clone)]
pub struct InputsExample {
    my_text: String,
}

impl InputsExample {
    const DROPDOWN_ITEMS: [&'static str; 4] =
        ["Dropdown Item 1", "Dropdown Item 2", "Dropdown Item 3", "Dropdown Item 4"];
}

impl Component for InputsExample {
    type Props = ();

    fn view_with_no_global_state(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .padding("20px", "20px", "20px", "20px")
            .display(Display::Block)
            .push(Text::new("Common Input Elements:").font_size(24.0))
            .push(TextInput::new(state.my_text.as_str()).id("text_input").margin("10px", "0px", "0px", "0px"))
            .push(Switch::new().default_toggled(true).margin("10px", "0px", "0px", "0px"))
            .push(
                Dropdown::new()
                    .push(Text::new(Self::DROPDOWN_ITEMS[0]))
                    .push(Text::new(Self::DROPDOWN_ITEMS[1]))
                    .push(Text::new(Self::DROPDOWN_ITEMS[2]))
                    .push(Text::new(Self::DROPDOWN_ITEMS[3]))
                    .margin("10px", "0px", "0px", "0px"),
            )
            .component()
    }

    fn update_with_no_global_state(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        if let CraftMessage(TextInputChanged(str)) = event.message {
            state.my_text = str.clone();
            return UpdateResult::new().prevent_defaults().prevent_propagate();
        }

        UpdateResult::new()
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();

    craft_main_with_options(
        InputsExample::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "inputs".to_string(),
        }),
    );
}
