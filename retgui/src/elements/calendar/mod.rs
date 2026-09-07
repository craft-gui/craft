//! A calendar.

use std::collections::VecDeque;
use std::sync::Arc;

use retgui_calendar::sys_locale::get_locale_or_default;
use retgui_calendar::{DateAddOptions, DateDuration, Locale, Month, Weekday, current_calendar_start, current_month, day_abbreviation, first_day_of_week, format_date_day_number, month_name, year_name};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{Container, ContainerElement, Dropdown, DropdownElement, DynElement, Element, ElementIds, ElementInternals, ElementStates, RetGuiAccessTree, RetainedElements, Text, TextElement};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use crate::text::text_context::TextContext;
use crate::{App, px, rgb};

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
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |_, _| None,
        ))
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
        elements: &RetainedElements,
        states: &ElementStates,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(
            self,
            elements,
            states,
            renderer,
            resource_manager,
            text_context,
            scale_factor,
        );
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        _event_queue: &mut VecDeque<EventKind>,
        _focus: &mut Option<DynElement>,
        _focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        _states: &mut ElementStates,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        let year_id = elements.get(self.year_dropdown.inner).element_data().internal_id;
        let month_id = elements.get(self.month_dropdown.inner).element_data().internal_id;
        if let EventKind::DropdownItemSelected(dropdown_event) = event {
            let target_id = elements.get(dropdown_event.target()).element_data().internal_id;
            if target_id == year_id {
                self.select_year(elements, gummy_tree, dropdown_event.index);
            } else if target_id == month_id {
                self.select_month(elements, gummy_tree, dropdown_event.index);
            }
        }
    }
}

impl Calendar {
    pub fn new(app: &mut App) -> Self {
        let App {
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            event_queue,
            focus,
            ..
        } = app;
        let locale = get_locale_or_default();
        let first_day = first_day_of_week(&locale);
        let start_of_month = current_month();
        let week_grid = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(week_grid.inner).set_display(gummy_tree, Display::Flex);
        elements
            .get_mut(week_grid.inner)
            .set_flex_direction(gummy_tree, FlexDirection::Column);
        let day_header = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        let nav = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        let year_dropdown = Dropdown {
            inner: DropdownElement::insert(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(year_dropdown.inner).set_width(gummy_tree, px(100));
        let month_dropdown = Dropdown {
            inner: DropdownElement::insert(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(month_dropdown.inner).set_width(gummy_tree, px(100));
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
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
                end_year: MAX_YEAR,
                locale,
            })
        });
        elements
            .get_mut(inner)
            .element_data_mut()
            .create_layout_node(gummy_tree, None);
        elements.dispatch_mut(inner, |inner_value, elements| {
            let inner = (inner_value as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap();
            inner.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
            inner.setup_months(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
            let mut current_header_day = inner.first_day;
            for _ in 0..COLUMNS {
                let label = day_abbreviation(&inner.locale, current_header_day);
                let text = Text {
                    inner: TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, label.as_str()),
                };
                elements.get_as_mut::<TextElement>(text.inner).set_selectable(false);
                let day = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
                elements.get_mut(day).set_display(gummy_tree, Display::Flex);
                elements
                    .get_mut(day)
                    .set_justify_content(gummy_tree, JustifyContent::Center);
                elements.get_mut(day).set_align_items(gummy_tree, AlignItems::Center);
                push_child_to_element(elements, gummy_tree, day, text.inner);
                elements.get_mut(day).set_width(gummy_tree, CELL_SIZE);
                elements.get_mut(day).set_height(gummy_tree, CELL_SIZE);
                push_child_to_element(elements, gummy_tree, inner.day_header.inner, day);
                current_header_day = Weekday::from_days_since_sunday(current_header_day as isize + 1);
            }
            for _ in 0..ROWS {
                let week = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
                elements.get_mut(week).set_display(gummy_tree, Display::Flex);
                elements
                    .get_mut(week)
                    .set_flex_direction(gummy_tree, FlexDirection::Row);
                for _ in 0..COLUMNS {
                    let text = Text {
                        inner: TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, ""),
                    };
                    elements.get_as_mut::<TextElement>(text.inner).set_selectable(false);
                    let day = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
                    elements
                        .get_mut(day)
                        .set_justify_content(gummy_tree, JustifyContent::Center);
                    elements.get_mut(day).set_align_items(gummy_tree, AlignItems::Center);
                    elements.get_mut(day).set_width(gummy_tree, CELL_SIZE);
                    elements.get_mut(day).set_height(gummy_tree, CELL_SIZE);
                    push_child_to_element(elements, gummy_tree, day, text.inner);
                    push_child_to_element(elements, gummy_tree, week, day);
                    inner.days.push(text);
                }
                push_child_to_element(elements, gummy_tree, inner.week_grid.inner, week);
            }
            inner.update_calendar(elements, gummy_tree);
            inner.set_display(gummy_tree, Display::Flex);
            inner.set_flex_direction(gummy_tree, FlexDirection::Column);
            let nav = elements.get_mut(inner.nav.inner);
            nav.set_display(gummy_tree, Display::Flex);
            nav.set_justify_content(gummy_tree, JustifyContent::SpaceAround);
            nav.set_align_items(gummy_tree, AlignItems::Center);
            nav.set_flex_direction(gummy_tree, FlexDirection::Row);
            nav.set_width(gummy_tree, px(CELL_SIZE.raw_value() * 7.0));
            push_child_to_element(elements, gummy_tree, inner.nav.inner, inner.year_dropdown.inner);
            push_child_to_element(elements, gummy_tree, inner.nav.inner, inner.month_dropdown.inner);
        });
        push_child_to_element(elements, gummy_tree, inner, nav.inner);
        push_child_to_element(elements, gummy_tree, inner, day_header.inner);
        push_child_to_element(elements, gummy_tree, inner, week_grid.inner);
        Self { inner }
    }

    pub fn set_start_year(&self, app: &mut App, year: i32) {
        if !app.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        app.elements.try_dispatch_mut(self.inner, |inner, arena| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_start_year(
                    arena,
                    &mut app.gummy_tree,
                    &app.access_tree,
                    &mut app.by_internal_id,
                    &mut app.event_queue,
                    &mut app.focus,
                    year,
                )
        });
    }

    pub fn set_end_year(&self, app: &mut App, year: i32) {
        if !app.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        app.elements.try_dispatch_mut(self.inner, |inner, arena| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_end_year(
                    arena,
                    &mut app.gummy_tree,
                    &app.access_tree,
                    &mut app.by_internal_id,
                    &mut app.event_queue,
                    &mut app.focus,
                    year,
                )
        });
    }
}

impl CalendarElement {
    fn update_calendar(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree) {
        let mut start_date = current_calendar_start(self.first_day, self.focus_year, Month::new(self.focus_month));
        for day_element in &self.days {
            let is_in_current_month = start_date.month().ordinal == self.focus_month;
            let date_str = format_date_day_number(&self.locale, &start_date);
            let day = elements.get_as_mut::<TextElement>(day_element.inner);
            day.set_text(gummy_tree, date_str.as_str());
            day.set_text_brush(
                gummy_tree,
                crate::Brush::Color(if is_in_current_month {
                    rgb(0, 0, 0)
                } else {
                    rgb(120, 120, 120)
                }),
            );
            start_date
                .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
                .unwrap()
        }
    }

    fn select_year(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, year: usize) {
        self.focus_year = self.end_year - (year as i32);
        self.update_calendar(elements, gummy_tree);
    }

    fn select_month(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, month: usize) {
        self.focus_month = 1 + month as u8;
        self.update_calendar(elements, gummy_tree);
    }

    fn setup_years(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
    ) {
        let dropdown = self.year_dropdown.inner;
        elements.dispatch_mut(dropdown, |dropdown, elements| {
            dropdown.remove_all_children(elements, gummy_tree, event_queue, focus);
        });
        for year in (self.start_year..(self.end_year + 1)).rev() {
            let text = TextElement::insert(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                &year_name(&self.locale, year),
            );
            elements.get_as_mut::<TextElement>(text).set_selectable(false);
            push_child_to_element(elements, gummy_tree, dropdown, text);
            elements.get_mut(dropdown).set_font_size(gummy_tree, 20.0);
            if year == self.focus_year {
                elements.dispatch_mut(dropdown, |dropdown, elements| {
                    (dropdown as &mut dyn std::any::Any)
                        .downcast_mut::<DropdownElement>()
                        .unwrap()
                        .set_selected_element(
                            elements,
                            gummy_tree,
                            access_tree,
                            by_internal_id,
                            (self.end_year - year) as usize,
                        );
                });
            }
        }
    }

    fn setup_months(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
    ) {
        let dropdown = self.month_dropdown.inner;
        elements.dispatch_mut(dropdown, |dropdown, elements| {
            dropdown.remove_all_children(elements, gummy_tree, event_queue, focus);
        });
        for month in 0..12 {
            let text = TextElement::insert(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                &month_name(&self.locale, Month::new(month + 1), self.focus_year),
            );
            elements.get_as_mut::<TextElement>(text).set_selectable(false);
            push_child_to_element(elements, gummy_tree, dropdown, text);
            elements.get_mut(dropdown).set_font_size(gummy_tree, 20.0);
            if month + 1 == self.focus_month {
                elements.dispatch_mut(dropdown, |dropdown, elements| {
                    (dropdown as &mut dyn std::any::Any)
                        .downcast_mut::<DropdownElement>()
                        .unwrap()
                        .set_selected_element(elements, gummy_tree, access_tree, by_internal_id, month as usize);
                });
            }
        }
    }

    pub fn set_start_year(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        year: i32,
    ) {
        if year > self.end_year {
            panic!("Invalid start year");
        }
        self.start_year = year;
        self.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
    }

    pub fn set_end_year(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        year: i32,
    ) {
        if year < self.start_year {
            panic!("Invalid end year");
        }
        self.end_year = year;
        self.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{DropdownItemSelectedEvent, EventDispatcher};

    #[test]
    fn calendar_rebuilds_years_and_updates_days_from_dropdown_events() {
        let mut app = App::new();
        let calendar = Calendar::new(&mut app);
        let year_dropdown = app.get_as::<CalendarElement>(calendar.inner).year_dropdown;
        let month_dropdown = app.get_as::<CalendarElement>(calendar.inner).month_dropdown;
        let old_years = year_dropdown.children(&app);

        calendar.set_end_year(&mut app, 2030);
        calendar.set_start_year(&mut app, 2020);
        assert_eq!(year_dropdown.children(&app).len(), 11);
        assert!(
            old_years
                .iter()
                .all(|year| app.contains(*year) && year.parent(&app).is_err())
        );

        for (dropdown, index) in [(year_dropdown, 6), (month_dropdown, 0)] {
            app.event_queue
                .push_back(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
                    dropdown.inner,
                    index,
                )));
        }
        EventDispatcher::dispatch_queued_events(&mut app);
        let calendar_element = app.get_as::<CalendarElement>(calendar.inner);
        assert_eq!((calendar_element.focus_year, calendar_element.focus_month), (2024, 1));
        let days = calendar_element.days.clone();
        assert_eq!(days.len(), ROWS * COLUMNS);
        let january: Vec<_> = days.iter().map(|day| day.text(&app)).collect();
        assert!(january.iter().all(|day| !day.is_empty()));

        app.event_queue
            .push_back(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
                month_dropdown.inner,
                1,
            )));
        EventDispatcher::dispatch_queued_events(&mut app);
        assert_eq!(app.get_as::<CalendarElement>(calendar.inner).focus_month, 2);
        assert_ne!(january, days.iter().map(|day| day.text(&app)).collect::<Vec<_>>());
    }
}
