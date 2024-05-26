use oku::application::Props;
use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::element::Element;
use oku::elements::text::Text;
use oku_core::events::EventResult;
use oku_core::reactive::reactive;
use oku_core::reactive::reactive::RUNTIME;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;
use std::sync::Arc;

struct Test1 {}

impl Component for Test1 {
    fn view(&self, _props: Option<&Props>, key: Option<String>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component<u64> for Hello {
    fn view(&self, props: Option<&Props>, key: Option<String>) -> Element {
        if RUNTIME.get_state::<u32>(0).is_none() {
            RUNTIME.set_state(0, 0u32);
        }

        let x: u32 = RUNTIME.get_state(0).unwrap();
        let container = Container::new().add_child(Element::Text(Text::new(format!("Hello, world! {}", x))));

        let mut custom_component = oku::elements::component::Component::new();
        custom_component = custom_component.add_child(Element::Container(container));

        custom_component.add_update_handler(Arc::new(|msg, state| {
            println!("msg: {:?}", 2);
            println!("state: {:?}", 2);

            let x: u32 = RUNTIME.get_state(0).unwrap();
            RUNTIME.set_state(0, x + 1);
        }));

        Element::Component(custom_component)
    }
}

struct App {}

impl oku_core::application::Application for App {
    fn view(&self) -> Element {
        let hello = Hello {};
        let hello_props = Props {
            data: Box::new(12_u32),
        };

        let test1 = Test1 {};

        hello.view(Some(&hello_props), None)
    }
}

fn main() {
    let application = App {};
    oku_core::oku_main_with_options(Box::new(application), Some(OkuOptions { renderer: Wgpu }));
}
