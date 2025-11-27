use craft_retained::elements::core::ElementData;
use craft_retained::elements::Element;
use craft_retained::elements::{Container, Text};
use craft_retained::events::ui_events::pointer::PointerButtonEvent;
use craft_retained::events::Event;
use craft_retained::style::Overflow;
use craft_retained::style::{Display, FlexDirection, Unit};
use craft_retained::Color;
use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use std::cell::RefCell;
use std::rc::Rc;

const ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

const COLOURS: &[&str] =
    &["red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black", "orange"];

const NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger", "pizza", "mouse",
    "keyboard",
];

#[derive(Clone)]
pub struct Data {
    id: usize,
    label: String,
}

impl Data {
    pub fn new(id: usize, label: String) -> Self {
        Self { id, label }
    }
}

pub struct State {
    store: Store,
    rows: Vec<Rc<RefCell<Container>>>,
    selected_row: Option<usize>,
    element: Rc<RefCell<dyn Element>>,
}

impl State {
    fn new(element: Rc<RefCell<dyn Element>>) -> Self {
        Self {
            store: Store::new(),
            rows: Vec::new(),
            selected_row: None,
            element,
        }
    }

    pub fn run(&mut self) {
        self.remove_all_rows();
        self.store.clear();
        self.rows.clear();
        self.store.run();
        self.append_rows();
        self.select(None);
    }

    pub fn run_lots(&mut self) {
        self.remove_all_rows();
        self.store.clear();
        self.rows.clear();
        self.store.run_lots();
        self.append_rows();
        self.select(None);
    }

    pub fn remove_all_rows(&mut self) {
        let to_remove = self.element.borrow().children().to_vec();
        for child in to_remove {
            self.element.borrow_mut().remove_child(child).expect("Failed to remove child!");
        }
    }

    pub fn swap_rows(&mut self) {
        if self.store.data.len() >= 999 {
            self.store.swap_rows();
            self.rows.swap(1, 998);
            let child_1 = self.element.borrow().children()[1].clone();
            let child_2 = self.element.borrow().children()[998].clone();
            self.element.borrow_mut().swap_child(child_1, child_2).expect("Failed to swap children");
        }
    }

    pub fn append_rows(&mut self) {
        for data in self.store.data.iter().skip(self.rows.len()) {
            let row = Self::create_row(data);
            self.rows.push(row.clone());
            self.element.borrow_mut().push_dyn(row);
        }
    }

    pub fn select(&mut self, row: Option<usize>) {
        self.selected_row = row;
    }

    pub fn create_row(data: &Data) -> Rc<RefCell<Container>> {
        let row = Container::new();
        row.borrow_mut().display(Display::Flex).push(Text::new(&data.id.to_string())).push(Text::new(&data.label));

        row
    }

    pub fn add(&mut self) {
        self.store.add();
        self.append_rows();
    }

    pub fn update(&mut self) {
        self.store.update();
        for (index, data) in self.store.data.iter().enumerate().step_by(10) {
            let container = self.rows[index].borrow_mut();
            let mut text = container.children()[1].borrow_mut();
            if let Some(text) = text.as_any_mut().downcast_mut::<Text>() {
                text.text(&data.label);
            }
        }
    }
}

pub struct Store {
    data: Vec<Data>,
    next_id: usize,
    rng: ThreadRng,
    selected: Option<usize>,
}

impl Store {
    pub fn swap_rows(&mut self) {
        if self.data.len() >= 999 {
            self.data.swap(1, 998);
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next_id: 1,
            rng: rng(),
            selected: None,
        }
    }

    pub fn build_data(&mut self, count: usize) {
        for _ in 0..count {
            self.data.push(Data::new(
                self.next_id,
                format!(
                    "{} {} {}",
                    ADJECTIVES.choose(&mut self.rng).unwrap(),
                    COLOURS.choose(&mut self.rng).unwrap(),
                    NOUNS.choose(&mut self.rng).unwrap()
                ),
            ));
            self.next_id += 1;
        }
    }

    pub fn run(&mut self) {
        self.build_data(1000);
        self.selected = None;
    }

    pub fn run_lots(&mut self) {
        self.build_data(10000);
        self.selected = None;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.selected = None;
    }

    pub fn select(&mut self, id: Option<usize>) {
        self.selected = id;
    }

    pub fn add(&mut self) {
        self.build_data(1000);
        self.selected = None;
    }

    pub fn delete(&mut self, id: usize) {
        self.data.retain(|f| f.id != id)
    }

    pub fn update(&mut self) {
        self.update_data();
        self.selected = None;
    }

    pub fn update_data(&mut self) {
        for data in self.data.iter_mut().step_by(10) {
            data.label += " !!!";
        }
    }
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    util::setup_logging();
    let root = build_root();
    use craft_retained::CraftOptions;
    craft_retained::craft_main(root, CraftOptions::basic("jsframeworkbench"));
}

fn build_root() -> Rc<RefCell<Container>> {
    let root = Container::new();
    let body = build_body();
    root.borrow_mut().push(body);
    root
}

fn build_body() -> Rc<RefCell<Container>> {
    let body = Container::new();
    let data_list = build_data_list();

    let state = Rc::new(RefCell::new(State::new(data_list.clone())));
    let buttons = build_buttons(state.clone());

    body.borrow_mut()
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .flex_direction(FlexDirection::Column);

    let text = Text::new(r#"Craft-"keyed""#);
    text.borrow_mut().font_size(64.0);

    body.borrow_mut().push(text).push(buttons).push(data_list);

    body
}

fn build_data_list() -> Rc<RefCell<Container>> {
    let data_list = Container::new();
    data_list.borrow_mut().flex_direction(FlexDirection::Column);
    data_list
}

fn build_buttons(state: Rc<RefCell<State>>) -> Rc<RefCell<Container>> {
    let buttons = Container::new();
    buttons.borrow_mut().flex_direction(FlexDirection::Column);

    let state1 = state.clone();
    let btn_create_1k = build_button("Create 1,000 rows", move |_, _| {
        state1.borrow_mut().run();
    });

    let state2 = state.clone();
    let btn_create_10k = build_button("Create 10,000 rows", move |_, _| {
        state2.borrow_mut().run_lots();
    });

    let state3 = state.clone();
    let btn_append_1k = build_button("Append 1,000 rows", move |_, _| state3.borrow_mut().add());

    let state4 = state.clone();
    let btn_update_10th_row = build_button("Update every 10th row", move |_, _| state4.borrow_mut().update());

    let state5 = state.clone();
    let btn_clear = build_button("Clear", move |_, _| state5.borrow_mut().remove_all_rows());

    let state6 = state.clone();
    let btn_swap = build_button("Swap Rows", move |_, _| state6.borrow_mut().swap_rows());

    buttons
        .borrow_mut()
        .push(btn_create_1k)
        .push(btn_create_10k)
        .push(btn_append_1k)
        .push(btn_update_10th_row)
        .push(btn_clear)
        .push(btn_swap);

    buttons
}

fn build_button<F>(label: &str, callback: F) -> Rc<RefCell<Container>>
where
    F: Fn(&mut Event, &PointerButtonEvent) + 'static,
{
    let button = Container::new();

    {
        let border_color = Color::from_rgb8(111, 111, 111);
        button
            .borrow_mut()
            .background_color(Color::from_rgb8(211, 211, 211))
            .border_color(border_color, border_color, border_color, border_color)
            .flex_direction(FlexDirection::Row)
            .flex_grow(0.0);
    }

    let text = Text::new(label);
    text.borrow_mut().selectable(false);
    button.borrow_mut().push(text);

    button.borrow_mut().on_pointer_button_up(Rc::new(callback));

    button
}
