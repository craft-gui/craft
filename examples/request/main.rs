mod ani_list;

use util::setup_logging;

use ani_list::{anime_view, AniListResponse, QUERY};
use AniListMessage::StateChange;

use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::{craft_main, WindowContext};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::style::FlexDirection;
use craft::style::{Display, Overflow, Unit, Wrap};
use craft::CraftOptions;

use reqwest::Client;
use serde_json::json;

use std::result::Result;
use craft::events::ui_events::pointer::PointerButtonUpdate;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    #[default]
    Initial,
    Loading,
    Loaded(AniListResponse),
    Error,
}

pub enum AniListMessage {
    StateChange(State),
}

#[derive(Default, Clone)]
pub struct AniList {
    state: State,
}

impl Component for AniList {
    type GlobalState = ();
    type Props = ();
    type Message = AniListMessage;

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext
    ) -> ComponentSpecification {
        let mut root = Container::new()
            .display(Display::Flex)
            .wrap(Wrap::Wrap)
            .width("100%")
            .height("100%")
            .overflow(Overflow::Scroll)
            .gap("40px")
            .padding(Unit::Px(20.0), Unit::Percentage(10.0), Unit::Px(20.0), Unit::Px(20.0))
            .push(
                Container::new()
                    .push(Text::new("Ani List Example").font_size(48.0).width("100%"))
                    .push(Text::new("Get Data").on_pointer_button_up(|state: &mut Self, _global_state: &mut Self::GlobalState, event: &mut Event, pointer_button: &PointerButtonUpdate| {
                        if state.state != State::Loading && pointer_button.is_primary() {
                            state.state = State::Loading;

                            let get_ani_list_data = async {
                                let client = Client::new();
                                let json = json!({"query": QUERY});

                                let response = client
                                    .post("https://graphql.anilist.co/")
                                    .header("Content-Type", "application/json")
                                    .header("Accept", "application/json")
                                    .body(json.to_string())
                                    .send()
                                    .await;

                                if let Err(response) = response {
                                    tracing::error!("Error fetching data: {:?}", response);
                                    return Event::async_result(StateChange(State::Error));
                                }

                                let result: Result<AniListResponse, reqwest::Error> = response.unwrap().json().await;

                                if let Err(response) = &result {
                                    tracing::error!("Error parsing data: {:?}", response);
                                    return Event::async_result(StateChange(State::Error));
                                }

                                let result = result.unwrap();
                                tracing::info!("Loaded data: ");
                                Event::async_result(StateChange(State::Loaded(result)))
                            };

                            event.future(get_ani_list_data);
                        }
                    }))
                    .width("100%")
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column),
            );

        match &self.state {
            State::Initial => {}
            State::Loading => {
                root = root.push(Text::new("Loading...").font_size(24.0));
            }
            State::Loaded(response) => {
                let mut anime_views = Vec::new();
                anime_views.extend(response.data.page.media.iter().map(anime_view));

                root = root.extend_children(anime_views);
            }
            State::Error => {
                root = root.push(Text::new("Error loading data").font_size(24.0));
            }
        }

        root.component()
    }

    fn on_user_message(&mut self, _global_state: &mut Self::GlobalState, _props: &Self::Props, _event: &mut Event, message: &Self::Message) {
        let StateChange(new_state) = message;
        self.state = new_state.clone();
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();
    craft_main(AniList::component(), (), CraftOptions::basic("Ani List"));
}
