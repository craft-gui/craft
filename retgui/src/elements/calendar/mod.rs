//! A calendar.

use std::sync::Arc;

use retgui_calendar::sys_locale::get_locale_or_default;
use retgui_calendar::{DateAddOptions, DateDuration, Locale, Month, Weekday, current_calendar_start, current_month, day_abbreviation, first_day_of_week, format_date_day_number, month_name, year_name};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{Container, Dropdown, DynElement, Element, ElementInternals, Elements, Text};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use crate::text::text_context::TextContext;
use crate::{px, rgb};

#[derive(Clone, Copy)]
pub struct Calendar {
    pub(crate) inner: DynElement,
}

/// A calendar.
#[derive(Clone)]
pub(crate) struct CalendarElement {
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

impl Element for Calendar {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for CalendarElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CalendarElement {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |_, _| None))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &self,
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        let year_id = elements.get(self.year_dropdown.inner).element_data().internal_id;
        let month_id = elements.get(self.month_dropdown.inner).element_data().internal_id;
        if let EventKind::DropdownItemSelected(dropdown_event) = event {
            let target_id = elements.get(dropdown_event.target()).element_data().internal_id;
            if target_id == year_id {
                self.select_year(elements, dropdown_event.index);
            } else if target_id == month_id {
                self.select_month(elements, dropdown_event.index);
            }
        }
    }
}

impl Calendar {
    pub fn new(elements: &mut Elements) -> Self {
        let locale = get_locale_or_default();
        let first_day = first_day_of_week(&locale);
        let start_of_month = current_month();
        let week_grid = Container::new(elements);
        week_grid.set_display(elements, Display::Flex);
        week_grid.set_flex_direction(elements, FlexDirection::Column);
        let day_header = Container::new(elements);
        let nav = Container::new(elements);
        let year_dropdown = Dropdown::new(elements);
        year_dropdown.set_width(elements, px(100));
        let month_dropdown = Dropdown::new(elements);
        month_dropdown.set_width(elements, px(100));
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(CalendarElement {
                element_data: ElementData::new(me, true, access_tree),
                week_grid,
                days: Vec::new(),
                focus_year: start_of_month.year().extended_year(),
                day_header,
                first_day,
                nav,
                focus_month: start_of_month.month().ordinal,
                year_dropdown,
                month_dropdown,
                start_year: MIN_YEAR,
                end_year: start_of_month.year().extended_year() + 2,
                locale,
            })
        });
        elements.create_layout_node(inner, None);
        elements.dispatch_mut(inner, |inner_value, elements| {
            let inner = (inner_value as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap();
            inner.setup_years(elements);
            inner.setup_months(elements);
            let mut current_header_day = inner.first_day;
            for _ in 0..COLUMNS {
                let label = day_abbreviation(&inner.locale, current_header_day);
                let text = Text::new(elements, label.as_str());
                text.set_selectable(elements, false);
                let day = Container::new(elements);
                day.set_display(elements, Display::Flex);
                day.set_justify_content(elements, JustifyContent::Center);
                day.set_align_items(elements, AlignItems::Center);
                day.push(elements, text);
                day.set_width(elements, CELL_SIZE);
                day.set_height(elements, CELL_SIZE);
                inner.day_header.push(elements, day);
                current_header_day = Weekday::from_days_since_sunday(current_header_day as isize + 1);
            }
            for _ in 0..ROWS {
                let week = Container::new(elements);
                week.set_display(elements, Display::Flex);
                week.set_flex_direction(elements, FlexDirection::Row);
                for _ in 0..COLUMNS {
                    let text = Text::new(elements, "");
                    text.set_selectable(elements, false);
                    let day = Container::new(elements);
                    day.set_justify_content(elements, JustifyContent::Center);
                    day.set_align_items(elements, AlignItems::Center);
                    day.set_width(elements, CELL_SIZE);
                    day.set_height(elements, CELL_SIZE);
                    day.push(elements, text);
                    week.push(elements, day);
                    inner.days.push(text);
                }
                inner.week_grid.push(elements, week);
            }
            inner.update_calendar(elements);
            inner.set_display(&mut elements.gummy_tree, Display::Flex);
            inner.set_flex_direction(&mut elements.gummy_tree, FlexDirection::Column);
            inner.nav.set_display(elements, Display::Flex);
            inner.nav.set_justify_content(elements, JustifyContent::SpaceAround);
            inner.nav.set_align_items(elements, AlignItems::Center);
            inner.nav.set_flex_direction(elements, FlexDirection::Row);
            inner.nav.set_width(elements, px(CELL_SIZE.raw_value() * 7.0));
            inner.nav.push(elements, inner.year_dropdown);
            inner.nav.push(elements, inner.month_dropdown);
        });
        crate::elements::internal_helpers::push_child_to_element(elements, inner, nav.inner);
        crate::elements::internal_helpers::push_child_to_element(elements, inner, day_header.inner);
        crate::elements::internal_helpers::push_child_to_element(elements, inner, week_grid.inner);
        Self { inner }
    }

    pub fn set_start_year(&self, elements: &mut Elements, year: i32) {
        if !elements.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        elements.try_dispatch_mut(self.inner, |inner, elements| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_start_year(elements, year)
        });
    }

    pub fn set_end_year(&self, elements: &mut Elements, year: i32) {
        if !elements.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        elements.try_dispatch_mut(self.inner, |inner, elements| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_end_year(elements, year)
        });
    }
}

impl CalendarElement {
    fn update_calendar(&mut self, elements: &mut Elements) {
        let mut start_date = current_calendar_start(self.first_day, self.focus_year, Month::new(self.focus_month));
        for day_element in &self.days {
            let is_in_current_month = start_date.month().ordinal == self.focus_month;
            let date_str = format_date_day_number(&self.locale, &start_date);
            day_element.set_text(elements, date_str.as_str());
            day_element.set_color(
                elements,
                if is_in_current_month {
                    rgb(0, 0, 0)
                } else {
                    rgb(120, 120, 120)
                },
            );
            start_date
                .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
                .unwrap()
        }
    }

    fn select_year(&mut self, elements: &mut Elements, year: usize) {
        self.focus_year = self.end_year - (year as i32);
        self.update_calendar(elements);
    }

    fn select_month(&mut self, elements: &mut Elements, month: usize) {
        self.focus_month = 1 + month as u8;
        self.update_calendar(elements);
    }

    fn setup_years(&mut self, elements: &mut Elements) {
        let dropdown = self.year_dropdown;
        dropdown.remove_all_children(elements);
        for year in (self.start_year..(self.end_year + 1)).rev() {
            let text = Text::new(elements, &year_name(&self.locale, year));
            text.set_selectable(elements, false);
            dropdown.push(elements, text);
            dropdown.set_font_size(elements, 20.0);
            if year == self.focus_year {
                dropdown.set_selected_item(elements, (self.end_year - year) as usize);
            }
        }
    }

    fn setup_months(&mut self, elements: &mut Elements) {
        let dropdown = self.month_dropdown;
        dropdown.remove_all_children(elements);
        for month in 0..12 {
            let text = Text::new(
                elements,
                &month_name(&self.locale, Month::new(month + 1), self.focus_year),
            );
            text.set_selectable(elements, false);
            dropdown.push(elements, text);
            dropdown.set_font_size(elements, 20.0);
            if month + 1 == self.focus_month {
                dropdown.set_selected_item(elements, month as usize);
            }
        }
    }

    pub fn set_start_year(&mut self, elements: &mut Elements, year: i32) {
        if year > self.end_year {
            panic!("Invalid start year");
        }
        self.start_year = year;
        self.setup_years(elements);
    }

    pub fn set_end_year(&mut self, elements: &mut Elements, year: i32) {
        if year < self.start_year {
            panic!("Invalid end year");
        }
        self.end_year = year;
        self.setup_years(elements);
    }
}
