use std::collections::VecDeque;
use std::sync::Arc;

use retgui::elements::{DynElement, Element, ElementData, ElementEditor, ElementIds, ElementInternals, ElementStates, HasElementData, RetGuiAccessTree, RetainedElements, Text, Window, clone_element};
use retgui::events::EventKind;
use retgui::layout::GummyTree;
use retgui::style::AlignSelf;
use retgui::text::text_context::TextContext;
use retgui::{App, Brush, Color, Renderer, ResourceManager, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

#[derive(Clone, Copy)]
struct ColorTile {
    inner: DynElement,
}

#[derive(Clone)]
struct ColorTileElement {
    element_data: ElementData,
    color: Color,
    alternate: Color,
    clicks: u32,
}

impl Element for ColorTile {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl HasElementData for ColorTileElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ColorTileElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        clone_element(self, elements, gummy_tree, access_tree, by_internal_id, |_, _| None)
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
        if !self.is_visible() {
            return;
        }

        self.add_hit_testable(renderer, true, scale_factor);
        self.draw_borders(renderer, scale_factor);
        let bounds = self.computed_box().content_rectangle().scale(scale_factor);
        renderer.draw_rect(bounds, Brush::Color(self.color));
        self.draw_children(elements, states, renderer, resource_manager, scale_factor, text_context);
    }

    fn on_event(
        &mut self,
        _elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
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
        if matches!(event, EventKind::Click(_)) {
            std::mem::swap(&mut self.color, &mut self.alternate);
            self.clicks += 1;
            self.request_window_redraw();
        }
    }
}

impl ColorTile {
    fn new(app: &mut App) -> Self {
        let inner = app.insert_element(true, |element_data| ColorTileElement {
            element_data,
            color: rgb(37, 99, 235),
            alternate: rgb(219, 39, 119),
            clicks: 0,
        });
        Self { inner }
    }

    fn fill_color(self, app: &mut App, color: Color) -> Self {
        let tile = app.get_as_mut::<ColorTileElement>(self.inner);
        tile.color = color;
        tile.request_window_redraw();
        self
    }

    fn click_count(self, app: &App) -> u32 {
        app.get_as::<ColorTileElement>(self.inner).clicks
    }
}

trait ColorTileEditorExt: Sized {
    fn fill_color(self, color: Color) -> Self;
}

impl ColorTileEditorExt for ElementEditor<'_, ColorTile> {
    fn fill_color(self, color: Color) -> Self {
        self.apply(|tile, app| {
            tile.fill_color(app, color);
        })
    }
}

fn main() {
    setup_logging();

    let mut app = App::new();
    let label = Text::new(&mut app, "Click the custom Element")
        .edit(&mut app)
        .font_size(20.0)
        .selectable(false)
        .finish();

    let tile = ColorTile::new(&mut app)
        .edit(&mut app)
        .fill_color(rgb(37, 99, 235))
        .align_self(AlignSelf::Start)
        .width(px(320))
        .height(px(180))
        .padding_all(px(24))
        .border_radius_all((16.0, 16.0))
        .push(label)
        .finish();

    assert_eq!(tile.click_count(&app), 0);

    Window::new(&mut app, "Custom Element")
        .edit(&mut app)
        .width(pct(100))
        .height(pct(100))
        .push(tile)
        .finish();

    retgui_main(app, RetGuiOptions::basic("Custom Element"));
}
