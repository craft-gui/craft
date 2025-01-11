mod ani_list;

use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, Text};
use oku::events::{Event, Message};
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku::{PinnedFutureAny, RendererType};

use reqwest::Client;

use oku::elements::ElementStyles;
use oku::style::{Display, Overflow, Unit, Wrap};
use serde_json::json;

use std::any::Any;
use std::result::Result;

#[derive(Debug, Clone, Default)]
#[derive(PartialEq)]
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

    fn view(state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {
        let mut root = Container::new()
            .display(Display::Flex)
            .wrap(Wrap::Wrap)
            .height("100%")
            .overflow(Overflow::Scroll)
            .background(Color::rgba(230, 230, 230, 255))
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
                anime_views.extend(response.data.page.media.iter().map(|media| anime_view(media)));

                root = root.extend_children(anime_views);
            }
            State::Error => {
                root = root.push(Text::new("Error loading data").font_size(24.0));
            }
        }

        root.component()
    }

    fn update(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        match event.message {
            Message::OkuMessage(_) => {}
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

        if state.state != State::Loading && clicked(&event.message) && Some("get_data") == event.target.as_deref() {
            state.state = State::Loading;
            return UpdateResult::default().future(get_ani_list_data);
        }

        UpdateResult::default()
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        AniList::component(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "Ani List".to_string(),
        }),
    );
}

use crate::ani_list::{anime_view, AniListResponse, QUERY};
use crate::AniListMessage::StateChange;
use oku::events::clicked;
#[cfg(target_os = "android")]
use oku::AndroidApp;
use oku::Color;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        AniList::component(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "Ani List".to_string(),
        }),
        app,
    );
}
