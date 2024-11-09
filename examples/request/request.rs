use oku::components::ComponentSpecification;
use oku::engine::events::Message;

use bytes::Bytes;
use oku::platform::resource_manager::ResourceIdentifier;
use oku::RendererType::Wgpu;
use oku::{oku_main_with_options, OkuOptions};
use oku::engine::events::OkuEvent;
use oku::components::{Component, ComponentId, UpdateResult};
use oku::elements::{Image, Container};
use oku::style::{Display, Unit, Wrap};
use oku::PinnedFutureAny;
use std::any::Any;

#[derive(Default, Clone)]
pub struct Request {
    image: Option<Vec<u8>>,
}

const RED_PANDA: &'static str =
    "https://upload.wikimedia.org/wikipedia/commons/thumb/e/e6/Red_Panda_%2824986761703%29.jpg/440px-Red_Panda_%2824986761703%29.jpg";
const TREE: &'static str = "https://www.w3schools.com/css/img_tree.png";

impl Component for Request {
    type Props = u64;

    fn view(
        state: &Self,
        _props: Option<&Self::Props>,
        _children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .wrap(Wrap::Wrap)
            .component()
            .push(
                Image::new(ResourceIdentifier::Url(RED_PANDA.to_string()))
                    .max_width(Unit::Percentage(100.0))
                    .display(Display::Block)
                    .component()
                    .key("counter text"),
            )
            .push(
                Image::new(ResourceIdentifier::Url(TREE.to_string()))
                    .max_width(Unit::Percentage(100.0))
                    .component()
                    .key("counter text"),
            )
    }

    fn update(state: &mut Self, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }
        /*
                let res: Option<PinnedFutureAny> = match message {
                    Message::OkuMessage(OkuEvent::PointerButtonEvent(pointer_button)) => Some(Box::pin(async {
                        let res = reqwest::get("https://picsum.photos/800").await;
                        let bytes = res.unwrap().bytes().await.ok();
                        let boxed: Box<dyn Any + Send> = Box::new(bytes);

                        boxed
                    })),
                    Message::UserMessage(user_message) => {
                        if let Ok(image_data) = user_message.downcast::<Option<Bytes>>() {
                            std::fs::write("a.jpg", image_data.clone().unwrap().as_ref()).ok();
                            state.image = Some(image_data.clone().unwrap().as_ref().to_vec());
                        }
                        None
                    }
                    _ => None,
                };
        */

        UpdateResult::new(false, None)
    }
}

fn main() {
    oku_main_with_options(
        Container::new().wrap(Wrap::Wrap).component().push(Request::component()),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "Request".to_string(),
        }),
    );
}
