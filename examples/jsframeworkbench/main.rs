use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

use retgui::elements::{Button, Container, Element, Elements, State as StateHandle, Text, Window};
use retgui::events::ClickEvent;
use retgui::palette::css::WHITE;
use retgui::style::{AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Unit};
use retgui::{Color, rgb};

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
    rows: Vec<Row>,
    selected_row: Option<usize>,
    element: Container,
}

#[derive(Clone)]
struct Row {
    element: Container,
    label: Text,
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

    fn prepare_run(&mut self, lots: bool) -> (Container, Vec<Data>) {
        self.store.clear();
        self.rows.clear();
        if lots {
            self.store.run_lots();
        } else {
            self.store.run();
        }
        self.selected_row = None;
        (self.element, std::mem::take(&mut self.store.data))
    }

    fn prepare_append(&mut self) -> (Container, Vec<Data>) {
        let old_len = self.rows.len();
        self.store.add();
        self.selected_row = None;
        (self.element, self.store.data.split_off(old_len))
    }

    fn finish_rows(&mut self, data: Vec<Data>, rows: Vec<Row>) {
        self.store.data.extend(data);
        self.rows.extend(rows);
    }

    fn prepare_swap(&mut self) -> Option<(Container, Container, Container)> {
        if self.store.data.len() >= 999 {
            self.store.swap_rows();
            self.rows.swap(1, 998);
            Some((self.element, self.rows[1].element, self.rows[998].element))
        } else {
            None
        }
    }

    fn prepare_clear(&mut self) -> Container {
        self.store.clear();
        self.rows.clear();
        self.selected_row = None;
        self.element
    }

    fn create_row(elements: &mut Elements, data: &Data) -> Row {
        let label = Text::new(elements, &data.label);
        let id = Text::new(elements, &data.id.to_string())
            .edit(elements)
            .width(Unit::Px(60.0))
            .margin(Unit::Px(0.0), Unit::Px(12.0), Unit::Px(0.0), Unit::Px(0.0))
            .finish();
        let element = Container::new(elements)
            .edit(elements)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .width(Unit::Auto)
            .padding(Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0))
            .border_color_all(Color::from_rgb8(230, 230, 230))
            .push(id)
            .push(label)
            .finish();
        Row { element, label }
    }

    fn prepare_update(&mut self) -> Vec<(Text, String)> {
        self.store.update();
        self.selected_row = None;
        self.store
            .data
            .iter()
            .enumerate()
            .step_by(10)
            .map(|(index, data)| (self.rows[index].label, data.label.clone()))
            .collect()
    }
}

fn attach_rows(elements: &mut Elements, state: StateHandle<State>, element: Container, data: Vec<Data>, replace: bool) {
    if replace {
        element.edit(elements).delete_all_children().finish();
    }

    let rows: Vec<Row> = data.iter().map(|data| State::create_row(elements, data)).collect();
    let mut editor = element.edit(elements);
    for row in &rows {
        editor = editor.push(row.element);
    }
    editor.finish();

    state.update(elements, |state| state.finish_rows(data, rows));
}

fn rebuild_rows(elements: &mut Elements, state: StateHandle<State>, lots: bool) {
    let (element, data) = state.update(elements, |state| state.prepare_run(lots));
    attach_rows(elements, state, element, data, true);
}

fn append_rows(elements: &mut Elements, state: StateHandle<State>) {
    let (element, data) = state.update(elements, State::prepare_append);
    attach_rows(elements, state, element, data, false);
}

fn clear_rows(elements: &mut Elements, state: StateHandle<State>) {
    let element = state.update(elements, State::prepare_clear);
    element.edit(elements).delete_all_children().finish();
}

fn swap_rows(elements: &mut Elements, state: StateHandle<State>) {
    if let Some((element, child_1, child_2)) = state.update(elements, State::prepare_swap) {
        element
            .swap_child(elements, child_1.as_dyn_element(), child_2.as_dyn_element())
            .expect("failed to swap rows");
    }
}

fn update_rows(elements: &mut Elements, state: StateHandle<State>) {
    let updates = state.update(elements, State::prepare_update);
    for (label, text) in updates {
        label.edit(elements).text(&text).finish();
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

    let mut elements = Elements::new();
    let data_list = build_data_list(&mut elements);
    let state = elements.insert_state(State::new(data_list));

    let body = build_body(&mut elements, state);
    Window::new(&mut elements, "JsFrameworkBench")
        .edit(&mut elements)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .push(body)
        .finish();

    use retgui::RetGuiOptions;
    retgui::retgui_main(elements, RetGuiOptions::basic("jsframeworkbench"));
}

fn build_body(elements: &mut Elements, state: StateHandle<State>) -> Container {
    let buttons = build_buttons(elements, state);

    let body = Container::new(elements)
        .edit(elements)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Start)
        .padding_all(Unit::Px(15.0))
        .finish();

    let text = Text::new(elements, r#"RetGui-"keyed""#)
        .edit(elements)
        .font_size(32.0)
        .color(Color::BLACK)
        .finish();

    let text_container = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .width(Unit::Percentage(50.0))
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .push(text)
        .finish();

    let header = Container::new(elements)
        .edit(elements)
        .background_color(rgb(238, 238, 238))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border_radius_all((6.0, 6.0))
        .padding(Unit::Px(10.0), Unit::Px(60.0), Unit::Px(10.0), Unit::Px(60.0))
        .push(text_container)
        .width(Unit::Percentage(100.0))
        .push(buttons)
        .finish();

    let data_list = state.read(elements).element;
    body.edit(elements).push(header).push(data_list).finish()
}

fn build_data_list(elements: &mut Elements) -> Container {
    Container::new(elements)
        .edit(elements)
        .flex_direction(FlexDirection::Column)
        .width(Unit::Percentage(100.0))
        .finish()
}

fn build_buttons(elements: &mut Elements, state: StateHandle<State>) -> Container {
    let buttons = Container::new(elements)
        .edit(elements)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::FlexEnd)
        .align_items(AlignItems::Start)
        .gap(Unit::Px(12.0), Unit::Px(12.0))
        .wrap(FlexWrap::Wrap)
        .max_height(Unit::Px(150.0))
        .finish();

    let btn_create_1k = build_button(elements, "Create 1,000 rows", move |_event, elements| {
        rebuild_rows(elements, state, false);
    });

    let btn_create_10k = build_button(elements, "Create 10,000 rows", move |_event, elements| {
        rebuild_rows(elements, state, true);
    });

    let btn_append_1k = build_button(elements, "Append 1,000 rows", move |_event, elements| {
        append_rows(elements, state);
    });
    let btn_update_10th_row = build_button(elements, "Update every 10th row", move |_event, elements| {
        update_rows(elements, state);
    });
    let btn_clear = build_button(elements, "Clear", move |_event, elements| {
        clear_rows(elements, state);
    });
    let btn_swap = build_button(elements, "Swap Rows", move |_event, elements| {
        swap_rows(elements, state);
    });

    buttons
        .edit(elements)
        .push(btn_create_1k)
        .push(btn_create_10k)
        .push(btn_append_1k)
        .push(btn_update_10th_row)
        .push(btn_clear)
        .push(btn_swap)
        .finish()
}

fn build_button<F>(elements: &mut Elements, label: &str, callback: F) -> Button
where
    F: Fn(&mut ClickEvent, &mut Elements) + 'static,
{
    let label = Text::new(elements, label)
        .edit(elements)
        .selectable(false)
        .color(Color::WHITE)
        .finish();
    Button::new(elements)
        .edit(elements)
        .background_color(Color::from_rgb8(211, 211, 211))
        .border_color_all(Color::from_rgb8(111, 111, 111))
        .flex_direction(FlexDirection::Row)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap(Unit::Px(12.0), Unit::Px(12.0))
        .width(Unit::Px(250.0))
        .height(Unit::Px(35.0))
        .background_color(Color::from_rgb8(51, 122, 183))
        .color(WHITE)
        .border_radius_all((4.0, 4.0))
        .push(label)
        .on_click(callback)
        .finish()
}
