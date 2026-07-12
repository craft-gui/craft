//! A calendar.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

use craft_calendar::sys_locale::get_locale_or_default;
use craft_calendar::{DateAddOptions, DateDuration, Locale, Month, Weekday, current_calendar_start, current_month, day_abbreviation, first_day_of_week, format_date_day_number, month_name, year_name};

use craft_primitives::geometry::{Affine, Point, Rectangle};

use craft_renderer::RenderList;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Container, Dropdown, Element, ElementInternals, Text, resolve_clip_for_scrollable};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit};
use crate::text::text_context::TextContext;
use crate::{px, rgb};

#[derive(Clone)]
pub struct Calendar {
    pub inner: Rc<RefCell<CalendarInner>>,
}

/// A calendar.
#[derive(Clone)]
pub struct CalendarInner {
    element_data: ElementData,
    pub first_day: Weekday,
    pub nav: Container,
    pub day_header: Container,
    pub week_grid: Container,
    pub days: Vec<Text>,
    pub year_dropdown: Dropdown,
    pub month_dropdown: Dropdown,
    pub focus_year: i32,
    pub focus_month: u8,
    pub start_year: i32,
    pub end_year: i32,
    pub locale: Locale,
}

const ROWS: usize = 6;
const COLUMNS: usize = 7;
const CELL_SIZE: Unit = Unit::Px(36.0);
const MIN_YEAR: i32 = 1900;
const MAX_YEAR: i32 = 3000;

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Calendar {}

impl Drop for CalendarInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Calendar {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl crate::elements::ElementData for CalendarInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CalendarInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        draw_generic_container(self, renderer, text_context, scale_factor);
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        _event: &mut Event,
        target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        let year_id = self.year_dropdown.borrow().element_data().internal_id;
        let month_id = self.month_dropdown.borrow().element_data().internal_id;
        if let EventKind::DropdownItemSelected(index) = message {
            let target_id = target.unwrap().borrow().element_data().internal_id;
            if target_id == year_id {
                self.select_year(*index);
            } else if target_id == month_id {
                self.select_month(*index);
            }
        }
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.apply_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Calendar {
    pub fn new() -> Self {
        let locale = get_locale_or_default();
        let first_day = first_day_of_week(&locale);
        let start_of_month = current_month();
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<CalendarInner>>| {
            RefCell::new(CalendarInner {
                element_data: ElementData::new(me.clone(), true),
                week_grid: Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column),
                days: Vec::new(),
                focus_year: start_of_month.year().extended_year(),
                day_header: Container::new(),
                first_day,
                nav: Container::new(),
                focus_month: start_of_month.month().ordinal,
                year_dropdown: Dropdown::new().width(px(100)),
                month_dropdown: Dropdown::new().width(px(100)),
                start_year: MIN_YEAR,
                end_year: start_of_month.year().extended_year() + 2,
                locale,
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.setup_years();
        inner_mut.setup_months();
        inner_mut.element_data.create_layout_node(None);

        let mut current_header_day = inner_mut.first_day;
        for _ in 0..COLUMNS {
            let day = day_abbreviation(&inner_mut.locale, current_header_day);
            inner_mut.day_header.clone().push(
                Container::new()
                    .display(Display::Flex)
                    .justify_content(Some(JustifyContent::Center))
                    .align_items(Some(AlignItems::Center))
                    .push(Text::new(day.as_str()).selectable(false))
                    .width(CELL_SIZE)
                    .height(CELL_SIZE),
            );
            current_header_day = Weekday::from_days_since_sunday(current_header_day as isize + 1)
        }
        for _ in 0..ROWS {
            let mut week = Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row);
            for _ in 0..COLUMNS {
                let day_text = Text::new("").selectable(false);
                let day = Container::new()
                    .justify_content(Some(JustifyContent::Center))
                    .align_items(Some(AlignItems::Center))
                    .width(CELL_SIZE)
                    .height(CELL_SIZE)
                    .push(day_text.clone());
                week = week.push(day.clone());
                inner_mut.days.push(day_text);
            }
            inner_mut.week_grid.clone().push(week);
        }
        inner_mut.update_calendar();

        inner_mut.set_display(Display::Flex);
        inner_mut.set_flex_direction(FlexDirection::Column);

        let nav = inner_mut.nav.clone();
        nav.clone()
            .display(Display::Flex)
            .justify_content(Some(JustifyContent::SpaceAround))
            .align_items(Some(AlignItems::Center))
            .flex_direction(FlexDirection::Row)
            .push(inner_mut.year_dropdown.clone())
            .push(inner_mut.month_dropdown.clone());
        inner_mut.push(nav.inner);

        let day_header = inner_mut.day_header.clone();
        inner_mut.push(day_header.inner);

        let week_grid = inner_mut.week_grid.clone();
        inner_mut.push(week_grid.inner);

        drop(inner_mut);
        Self { inner }
    }

    pub fn start_year(self, year: i32) -> Self {
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        self.inner.borrow_mut().set_start_year(year);
        self
    }

    pub fn end_year(self, year: i32) -> Self {
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        self.inner.borrow_mut().set_end_year(year);
        self
    }
}

impl CalendarInner {
    fn update_calendar(&mut self) {
        let mut start_date = current_calendar_start(self.first_day, self.focus_year, Month::new(self.focus_month));
        for day_element in &self.days {
            let is_in_current_month = start_date.month().ordinal == self.focus_month;
            let date_str = format_date_day_number(&self.locale, &start_date);
            day_element
                .clone()
                .text(date_str.as_str())
                .color(if is_in_current_month {
                    rgb(0, 0, 0)
                } else {
                    rgb(120, 120, 120)
                });
            start_date
                .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
                .unwrap()
        }
    }

    fn select_year(&mut self, year: usize) {
        self.focus_year = self.end_year - (year as i32);
        self.update_calendar();
    }

    fn select_month(&mut self, month: usize) {
        self.focus_month = 1 + month as u8;
        self.update_calendar();
    }

    fn setup_years(&mut self) {
        let dropdown = self.year_dropdown.clone();
        dropdown.remove_all_children();
        for year in (self.start_year..(self.end_year + 1)).rev() {
            dropdown
                .clone()
                .push(Text::new(&year_name(&self.locale, year)))
                .font_size(20.0);
            if year == self.focus_year {
                dropdown.clone().selected_item((self.end_year - year) as usize);
            }
        }
    }

    fn setup_months(&mut self) {
        let dropdown = self.month_dropdown.clone();
        dropdown.remove_all_children();
        for month in 0..12 {
            dropdown
                .clone()
                .push(Text::new(&month_name(&self.locale, Month::new(month + 1), self.focus_year)))
                .font_size(20.0);
            if month + 1 == self.focus_month {
                dropdown.clone().selected_item(month as usize);
            }
        }
    }

    pub fn set_start_year(&mut self, year: i32) {
        if year > self.end_year {
            panic!("Invalid start year");
        }
        self.start_year = year;
        self.setup_years();
    }

    pub fn set_end_year(&mut self, year: i32) {
        if year < self.start_year {
            panic!("Invalid end year");
        }
        self.end_year = year;
        self.setup_years();
    }
}
