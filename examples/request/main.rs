mod ani_list;

use util::{setup_logging, ExampleProps};

use ani_list::{anime_view, AniListResponse, QUERY};
use AniListMessage::StateChange;

use craft::components::{Component, ComponentSpecification, Context, Event};
use craft::{craft_main, palette, Color};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::style::FlexDirection;
//noinspection ALL
use craft::style::{Display, Overflow, Wrap};
use craft::CraftOptions;

use reqwest::Client;
use serde_json::json;

use craft::events::ui_events::pointer::PointerButtonUpdate;
use std::result::Result;

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

fn fetch_trending_anime(context: &mut Context<AniList>, pointer_button: &PointerButtonUpdate) { {
        if context.state().state != State::Loading && pointer_button.is_primary() {
            context.state_mut().state = State::Loading;

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

                let result: Result<AniListResponse, reqwest::Error> =
                    response.unwrap().json().await;

                if let Err(response) = &result {
                    tracing::error!("Error parsing data: {:?}", response);
                    return Event::async_result(StateChange(State::Error));
                }

                let result = result.unwrap();
                tracing::info!("Loaded data: ");
                Event::async_result(StateChange(State::Loaded(result)))
            };

            context.event_mut().future(get_ani_list_data);
        }
    }
}

impl Component for AniList {
    type GlobalState = ();
    type Props = ExampleProps;
    type Message = AniListMessage;

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        
        let mut root_wrapper = Container::new()
            .overflow_y(if context.props().show_scrollbar { Overflow::Scroll } else { Overflow::Visible })
            .width("100%")
            .height("100%");
        
        let example_title = Text::new("AniList").font_size(32.0).width("100%");
        let fetch_trending_anime_button = Container::new()
            .push(Text::new("Show Trending")
            .display(Display::Flex)
            .color(Color::BLACK)
            .font_size(14.0)
            .padding("10px", "25px", "10px", "25px")
            .border_radius(4.0, 4.0, 4.0, 4.0)
            .border_width("2px", "2px", "2px", "2px")
            .border_color(palette::css::BLACK)
            .on_pointer_up(fetch_trending_anime));
        
        let mut root = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .max_width("1320px")
            .gap("10px")
            .margin("0px", "auto", "0px", "auto")
            .padding("20px", "20px", "20px", "20px")
            .push(example_title)
            .push(fetch_trending_anime_button);

        match &context.state().state {
            State::Initial => {}
            State::Loading => {
                root = root.push(Text::new("Loading...").font_size(24.0));
            }
            State::Loaded(response) => {
                let mut anime_views = Vec::new();
                anime_views.extend(response.data.page.media.iter().map(anime_view));

                let anime_wrapper = Container::new()
                    .margin("20px", "0px", "0px", "0px")
                    .display(Display::Flex)
                    .gap("30px")
                    .wrap(Wrap::Wrap)
                    .extend_children(anime_views);
                
                root = root.push(anime_wrapper);
            }
            State::Error => {
                root = root.push(Text::new("Error loading data").font_size(24.0));
            }
        }

        root_wrapper.push_in_place(root.component());
        root_wrapper.component()
    }

    fn on_user_message(context: &mut Context<Self>, message: &Self::Message) {
        let StateChange(new_state) = message;
        context.state_mut().state = new_state.clone();
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();
    craft_main(AniList::component(), (), CraftOptions::basic("Ani List"));
}
