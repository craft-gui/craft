use oku_core::components::component::Component;
use oku_core::elements::container::Container;
use oku_core::elements::element::Element;
use oku_core::elements::text::Text;
use oku_core::Props;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use oku_core::widget_id::{create_unique_widget_id, get_current_widget_id_counter};

fn use_state<T: Clone>(value: T) -> (impl Fn() -> T, impl FnMut(T)) {
    let val = Rc::new(RefCell::new(value));

    let state = {
        let val = val.clone();
        move || -> T { val.borrow().clone() }
    };

    let set_state = move |v: T| {
        val.replace(v);
    };

    (state, set_state)
}

struct Test1 {}

impl Component for Test1 {
    fn view(&self, props: Option<&Props>, children: Vec<Element>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component for Hello {
    fn view(&self, props: Option<&Props>, children: Vec<Element>) -> Element {
        let (data, mut set_data) = use_state(String::from("foo"));

        println!("data: {}", data());
        set_data(String::from("bar"));
        println!("data: {}", data());

        let my_data = props.unwrap().get_data::<u32>().unwrap();
        let mut container = Container::new().add_child(Element::Text(Text::new(format!("Hello, world! {}", my_data))));

        for child in children {
            container = container.add_child(child);
        }

        Element::Container(container)
    }
}

struct App {}

impl oku_core::Application for App {
    fn view(&self) -> Element {
        let hello = Hello {};
        let hello_props = Props {
            data: Box::new(12_u32),
        };

        let test1  = Test1 {};

        hello.view(Some(&hello_props), vec![
            test1.view(None, vec![])
        ])
    }
}

fn main() {
    let application = App {};
    oku_core::main(Box::new(application));
}
