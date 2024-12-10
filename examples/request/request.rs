mod ani_list;

use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, Text};
use oku_core::events::{ButtonSource, ElementState, Event, Message, MouseButton};
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use reqwest::Client;
use serde_json::json;
use std::any::Any;

use oku::elements::ElementStyles;
use oku_core::events::OkuMessage::PointerButtonEvent;
use oku_core::renderer::color::Color;
use oku::style::{Display, Overflow, Unit, Wrap};

#[derive(Default, Clone)]
pub struct AniList {
    pub(crate) response_data: Option<AniListResponse>,
}

impl Component for AniList {
    type Props = ();

    fn view(state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {

        let mut anime_views = Vec::new();
        if let Some(response) = &state.response_data {
            for media in response.data.page.media.clone() {
                anime_views.push(anime_view(&media));
            }
        }

        let mut root = Container::new()
            .display(Display::Flex)
            .wrap(Wrap::Wrap)
            .height("100%")
            .overflow(Overflow::Scroll)
            .background(Color::rgba(230, 230, 230, 255))
            .gap("40px")
            .padding(Unit::Px(20.0), Unit::Percentage(10.0), Unit::Px(20.0), Unit::Percentage(10.0))
            .push(Container::new()
                .push(Text::new("Ani List Example").font_size(48.0).width("100%"))
                .push(Text::new("Get Data").id("get_data"))
                .width("100%")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
            );

        for anime_view in anime_views {
            root = root.push(anime_view);
        }

        root.component()
    }

    fn update(state: &mut Self, _props: &Self::Props, message: Event) -> UpdateResult {
        match message.message {
            Message::OkuMessage(_) => {}
            Message::UserMessage(msg) => {
                if let Some(response) = msg.downcast_ref::<AniListResponse>() {
                    state.response_data = Some(response.clone());
                }
                return UpdateResult::default();
            }
        }

        let get_ani_list_data: PinnedFutureAny = Box::pin(async {
            let client = Client::new();
            let json = json!({"query": QUERY});

            let response = client.post("https://graphql.anilist.co/")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .body(json.to_string())
                .send()
                .await
                .unwrap()
                .text()
                .await;

            let result: AniListResponse = serde_json::from_str(&response.unwrap()).unwrap();

            let boxed: Box<dyn Any + Send> = Box::new(result);
            boxed
        });

        if let (Some("get_data"), Message::OkuMessage(PointerButtonEvent(pointer_button))) = (message.target.as_deref(), &message.message) {
            if pointer_button.button.mouse_button() == MouseButton::Left
                && pointer_button.state == ElementState::Released
            {
                return UpdateResult::default().future(get_ani_list_data);
            }
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
            renderer: Vello,
            window_title: "Ani List".to_string(),
        }),
    );
}

use crate::ani_list::{anime_view, AniListResponse, QUERY};
#[cfg(target_os = "android")]
use oku::AndroidApp;
use oku::PinnedFutureAny;
use oku::RendererType::Vello;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        AniList::component(),
        Some(OkuOptions {
            renderer: Vello,
            window_title: "Counter".to_string(),
        }),
        app,
    );
}
