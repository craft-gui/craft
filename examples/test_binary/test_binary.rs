use oku::application::Props;
use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::element::Element;
use oku::elements::text::Text;
use oku_core::events::EventResult;
use oku_core::reactive::reactive::Runtime;
use oku_core::reactive::use_state::{use_click, use_state};
use oku_core::OkuOptions;
use oku_core::RendererType::{Software, Wgpu};
use rand::random;
use std::cell::RefCell;
use std::rc::Rc;

/*fn use_state<T: Clone>(value: T) -> (impl Fn() -> T, impl FnMut(T)) {
    let val = Rc::new(RefCell::new(value));

    let state = {
        let val = val.clone();
        move || -> T { val.borrow().clone() }
    };

    let set_state = move |v: T| {
        val.replace(v);
    };

    (state, set_state)
}*/

struct Test1 {}

impl Component for Test1 {
    fn view(&self, _props: Option<&Props>, _children: Vec<Element>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component for Hello {
    fn view(&self, props: Option<&Props>, children: Vec<Element>) -> Element {
        let x = 5;
        let (number, mut set_number) = use_state(x);

        let num = number();

        use_click(Box::new(move |click: (u32, u32)| {
            set_number(num + 1);

            EventResult::Stop
        }));

        // let my_data = props.unwrap().get_data::<u32>().unwrap();
        let mut container = Container::new().add_child(Element::Text(Text::new(format!("Hello, world! {}", num.clone()))));

        for child in children {
            container = container.add_child(child);
        }

        Element::Container(container)
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

        hello.view(Some(&hello_props), vec![test1.view(None, vec![])])
    }
}

fn main() {
    let application = App {};
    oku_core::oku_main_with_options(Box::new(application), Some(OkuOptions { renderer: Wgpu }));
}
