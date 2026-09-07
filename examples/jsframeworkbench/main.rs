use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

use retgui::elements::{Button, Container, Element, State as StateHandle, Text, Window};
use retgui::events::ClickEvent;
use retgui::palette::css::WHITE;
use retgui::style::{AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Unit};
use retgui::{App, Color, rgb};

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

    fn create_row(app: &mut App, data: &Data) -> Row {
        let label = Text::new(app, &data.label);
        let id = Text::new(app, &data.id.to_string())
            .edit(app)
            .width(Unit::Px(60.0))
            .margin(Unit::Px(0.0), Unit::Px(12.0), Unit::Px(0.0), Unit::Px(0.0))
            .finish();
        let element = Container::new(app)
            .edit(app)
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

fn attach_rows(app: &mut App, state: StateHandle<State>, element: Container, data: Vec<Data>, replace: bool) {
    if replace {
        element.edit(app).delete_all_children().finish();
    }

    let rows: Vec<Row> = data.iter().map(|data| State::create_row(app, data)).collect();
    let mut editor = element.edit(app);
    for row in &rows {
        editor = editor.push(row.element);
    }
    editor.finish();

    state.update(app, |state| state.finish_rows(data, rows));
}

fn rebuild_rows(app: &mut App, state: StateHandle<State>, lots: bool) {
    let (element, data) = state.update(app, |state| state.prepare_run(lots));
    attach_rows(app, state, element, data, true);
}

fn append_rows(app: &mut App, state: StateHandle<State>) {
    let (element, data) = state.update(app, State::prepare_append);
    attach_rows(app, state, element, data, false);
}

fn clear_rows(app: &mut App, state: StateHandle<State>) {
    let element = state.update(app, State::prepare_clear);
    element.edit(app).delete_all_children().finish();
}

fn swap_rows(app: &mut App, state: StateHandle<State>) {
    if let Some((element, child_1, child_2)) = state.update(app, State::prepare_swap) {
        element
            .swap_child(app, child_1.as_dyn_element(), child_2.as_dyn_element())
            .expect("failed to swap rows");
    }
}

fn update_rows(app: &mut App, state: StateHandle<State>) {
    let updates = state.update(app, State::prepare_update);
    for (label, text) in updates {
        label.edit(app).text(&text).finish();
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

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    //util::setup_logging();

    let mut app = App::new();
    let data_list = build_data_list(&mut app);
    let state = app.insert_state(State::new(data_list));

    let body = build_body(&mut app, state);
    Window::new(&mut app, "JsFrameworkBench")
        .edit(&mut app)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .push(body)
        .finish();

    use retgui::RetGuiOptions;

    retgui::retgui_main(app, RetGuiOptions::basic("jsframeworkbench"));
}

fn build_body(app: &mut App, state: StateHandle<State>) -> Container {
    let buttons = build_buttons(app, state);

    let body = Container::new(app)
        .edit(app)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Start)
        .padding_all(Unit::Px(15.0))
        .finish();

    let text = Text::new(app, r#"RetGui-"keyed""#)
        .edit(app)
        .font_size(32.0)
        .color(Color::BLACK)
        .finish();

    let text_container = Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .width(Unit::Percentage(50.0))
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .push(text)
        .finish();

    let header = Container::new(app)
        .edit(app)
        .background_color(rgb(238, 238, 238))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border_radius_all((6.0, 6.0))
        .padding(Unit::Px(10.0), Unit::Px(60.0), Unit::Px(10.0), Unit::Px(60.0))
        .push(text_container)
        .width(Unit::Percentage(100.0))
        .push(buttons)
        .finish();

    let data_list = state.read(app).element;
    body.edit(app).push(header).push(data_list).finish()
}

fn build_data_list(app: &mut App) -> Container {
    Container::new(app)
        .edit(app)
        .flex_direction(FlexDirection::Column)
        .width(Unit::Percentage(100.0))
        .finish()
}

fn build_buttons(app: &mut App, state: StateHandle<State>) -> Container {
    let buttons = Container::new(app)
        .edit(app)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::FlexEnd)
        .align_items(AlignItems::Start)
        .gap(Unit::Px(12.0), Unit::Px(12.0))
        .wrap(FlexWrap::Wrap)
        .max_height(Unit::Px(150.0))
        .finish();

    let btn_create_1k = build_button(app, "Create 1,000 rows", move |_event, app| {
        rebuild_rows(app, state, false);
    });

    let btn_create_10k = build_button(app, "Create 10,000 rows", move |_event, app| {
        rebuild_rows(app, state, true);
    });

    let btn_append_1k = build_button(app, "Append 1,000 rows", move |_event, app| {
        append_rows(app, state);
    });
    let btn_update_10th_row = build_button(app, "Update every 10th row", move |_event, app| {
        update_rows(app, state);
    });
    let btn_clear = build_button(app, "Clear", move |_event, app| {
        clear_rows(app, state);
    });
    let btn_swap = build_button(app, "Swap Rows", move |_event, app| {
        swap_rows(app, state);
    });

    buttons
        .edit(app)
        .push(btn_create_1k)
        .push(btn_create_10k)
        .push(btn_append_1k)
        .push(btn_update_10th_row)
        .push(btn_clear)
        .push(btn_swap)
        .finish()
}

fn build_button<F>(app: &mut App, label: &str, callback: F) -> Button
where
    F: Fn(&mut ClickEvent, &mut App) + 'static,
{
    let label = Text::new(app, label)
        .edit(app)
        .selectable(false)
        .color(Color::WHITE)
        .finish();
    Button::new(app)
        .edit(app)
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
        .add_click_listener(callback)
        .finish()
}
