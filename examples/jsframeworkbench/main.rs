use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Text, TextInner, Window};
use craft_retained::events::Event;
use craft_retained::events::ui_events::pointer::PointerButtonEvent;
use craft_retained::palette::css::WHITE;
use craft_retained::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit, Wrap};
use craft_retained::{Color, rgb};
use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

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

const COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black", "orange",
];

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
    rows: Vec<Container>,
    selected_row: Option<usize>,
    element: Container,
}

impl State {
    fn new(element: Container) -> Self {
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
        let to_remove = self.element.get_children();
        for child in to_remove {
            self.element.remove_child(child).expect("Failed to remove child!");
        }
    }

    pub fn swap_rows(&mut self) {
        if self.store.data.len() >= 999 {
            self.store.swap_rows();
            self.rows.swap(1, 998);
            let child_1 = self.element.get_children()[1].clone();
            let child_2 = self.element.get_children()[998].clone();
            self.element
                .swap_child(child_1, child_2)
                .expect("Failed to swap children");
        }
    }

    pub fn append_rows(&mut self) {
        // Collect all new rows that need to be appended
        let new_rows: Vec<Container> = self
            .store
            .data
            .iter()
            .skip(self.rows.len())
            .map(|data| {
                let row = Self::create_row(data);
                row
            })
            .collect();

        self.rows.extend(new_rows.iter().cloned());

        for row in new_rows {
            self.element.clone().push(row);
        }
    }

    pub fn select(&mut self, row: Option<usize>) {
        self.selected_row = row;
    }

    pub fn create_row(data: &Data) -> Container {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .width(Unit::Auto)
            .padding(Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0))
            .border_color(
                Color::from_rgb8(230, 230, 230),
                Color::from_rgb8(230, 230, 230),
                Color::from_rgb8(230, 230, 230),
                Color::from_rgb8(230, 230, 230),
            )
            .push(Text::new(&data.id.to_string()).width(Unit::Px(60.0)).margin(
                Unit::Px(0.0),
                Unit::Px(12.0),
                Unit::Px(0.0),
                Unit::Px(0.0),
            ))
            .push(Text::new(&data.label))
    }

    pub fn add(&mut self) {
        self.store.add();
        self.append_rows();
    }

    pub fn clear(&mut self) {
        self.store.clear();
        self.rows.clear();
        self.remove_all_rows();
        self.select(None);
    }

    pub fn update(&mut self) {
        self.store.update();
        for (index, data) in self.store.data.iter().enumerate().step_by(10) {
            let container = self.rows[index].clone();
            let text = container.get_children()[1].clone();
            if let Some(text) = text.borrow_mut().as_any_mut().downcast_mut::<TextInner>() {
                text.set_text(&data.label);
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
        self.data.reserve(count);
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
    //util::setup_logging();

    let data_list = build_data_list();
    let state = Rc::new(RefCell::new(State::new(data_list.clone())));

    let body = build_body(state);
    Window::new()
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .push(body);

    use craft_retained::CraftOptions;
    craft_retained::craft_main(CraftOptions::basic("jsframeworkbench"));
}

fn build_body(state: Rc<RefCell<State>>) -> Container {
    let buttons = build_buttons(state.clone());

    let body = Container::new()
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .flex_direction(FlexDirection::Column)
        .align_items(Some(AlignItems::Start))
        .padding(Unit::Px(15.0), Unit::Px(15.0), Unit::Px(15.0), Unit::Px(15.0));

    let text = Text::new(r#"Craft-"keyed""#).font_size(32.0).color(Color::BLACK);

    let text_container = Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .width(Unit::Percentage(50.0))
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .push(text);

    let header = Container::new()
        .background_color(rgb(238, 238, 238))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border_radius((6.0, 6.0), (6.0, 6.0), (6.0, 6.0), (6.0, 6.0))
        .padding(Unit::Px(10.0), Unit::Px(60.0), Unit::Px(10.0), Unit::Px(60.0))
        .push(text_container)
        .width(Unit::Percentage(100.0))
        .push(buttons);

    body.push(header).push(state.borrow().element.clone())
}

fn build_data_list() -> Container {
    Container::new()
        .flex_direction(FlexDirection::Column)
        .width(Unit::Percentage(100.0))
}

fn build_buttons(state: Rc<RefCell<State>>) -> Container {
    let buttons = Container::new()
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::FlexEnd))
        .align_items(Some(AlignItems::Start))
        .gap(Unit::Px(12.0), Unit::Px(12.0))
        .wrap(Wrap::Wrap)
        .max_height(Unit::Px(150.0));

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
    let btn_clear = build_button("Clear", move |_, _| state5.borrow_mut().clear());

    let state6 = state.clone();
    let btn_swap = build_button("Swap Rows", move |_, _| state6.borrow_mut().swap_rows());

    buttons
        .push(btn_create_1k)
        .push(btn_create_10k)
        .push(btn_append_1k)
        .push(btn_update_10th_row)
        .push(btn_clear)
        .push(btn_swap)
}

fn build_button<F>(label: &str, callback: F) -> Container
where
    F: Fn(&mut Event, &PointerButtonEvent) + 'static,
{
    let border_color = Color::from_rgb8(111, 111, 111);
    Container::new()
        .background_color(Color::from_rgb8(211, 211, 211))
        .border_color(border_color, border_color, border_color, border_color)
        .flex_direction(FlexDirection::Row)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .gap(Unit::Px(12.0), Unit::Px(12.0))
        .width(Unit::Px(250.0))
        .height(Unit::Px(35.0))
        .background_color(Color::from_rgb8(51, 122, 183))
        .color(WHITE)
        .border_radius((4.0, 4.0), (4.0, 4.0), (4.0, 4.0), (4.0, 4.0))
        .push(Text::new(label).selectable(false).color(Color::WHITE))
        .on_pointer_button_up(Rc::new(callback))
}
