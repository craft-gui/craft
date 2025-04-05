mod ani_list;

#[path = "../util.rs"]
mod util;

use ani_list::{anime_view, AniListResponse, QUERY};
use util::setup_logging;
use AniListMessage::StateChange;

use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::events::{Event, Message};
use craft::craft_main_with_options;
use craft::style::FlexDirection;
use craft::style::{Display, Overflow, Unit, Wrap};
use craft::Color;
use craft::CraftOptions;
use craft::RendererType;

use reqwest::Client;
use serde_json::json;

use std::result::Result;

#[derive(Debug, Clone, Default, PartialEq)]
enum State {
    #[default]
    Initial,
    Loading,
    Loaded(AniListResponse),
    Error,
}

enum AniListMessage {
    StateChange(State),
}

#[derive(Default, Clone)]
pub struct AniList {
    state: State,
}

impl Component for AniList {
    type Props = ();

    fn view_with_no_global_state(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        let mut root = Container::new()
            .display(Display::Flex)
            .wrap(Wrap::Wrap)
            .width("100%")
            .height("100%")
            .overflow(Overflow::Scroll)
            .background(Color::from_rgb8(230, 230, 230))
            .gap("40px")
            .padding(Unit::Px(20.0), Unit::Percentage(10.0), Unit::Px(20.0), Unit::Px(20.0))
            .push(
                Container::new()
                    .push(Text::new("Ani List Example").font_size(48.0).width("100%"))
                    .push(Text::new("Get Data").id("get_data"))
                    .width("100%")
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column),
            );

        match &state.state {
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

    fn update_with_no_global_state(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        match event.message {
            Message::CraftMessage(_) => {}
            Message::UserMessage(msg) => {
                if let Some(StateChange(new_state)) = msg.downcast_ref::<AniListMessage>() {
                    state.state = new_state.clone();
                }
                return UpdateResult::default();
            }
        }

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
                return UpdateResult::async_result(StateChange(State::Error));
            }

            let result: Result<AniListResponse, reqwest::Error> = response.unwrap().json().await;

            if let Err(response) = &result {
                tracing::error!("Error parsing data: {:?}", response);
                return UpdateResult::async_result(StateChange(State::Error));
            }

            let result = result.unwrap();
            tracing::info!("Loaded data: ");
            UpdateResult::async_result(StateChange(State::Loaded(result)))
        };

        if state.state != State::Loading && event.message.clicked() && Some("get_data") == event.target.as_deref() {
            state.state = State::Loading;
            return UpdateResult::default().future(get_ani_list_data);
        }

        UpdateResult::default()
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();

    craft_main_with_options(
        AniList::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Ani List".to_string(),
        }),
    );
}