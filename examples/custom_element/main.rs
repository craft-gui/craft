use std::sync::Arc;

use retgui::elements::{DynElement, Element, ElementData, ElementEditor, ElementNode, ElementNodeData, Elements, Text, Window, clone_element};
use retgui::events::EventKind;
use retgui::style::AlignSelf;
use retgui::text::text_context::TextContext;
use retgui::{Brush, Color, Renderer, ResourceManager, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

#[derive(Clone, Copy)]
struct ColorTile {
    inner: DynElement,
}

#[derive(Clone)]
struct ColorTileNode {
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

impl ElementNodeData for ColorTileNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for ColorTileNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        clone_element(self, elements, |_, _| None)
    }

    fn draw(
        &self,
        elements: &Elements,
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
        self.draw_children(elements, renderer, resource_manager, scale_factor, text_context);
    }

    fn on_event(&mut self, _elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        if matches!(event, EventKind::Click(_)) {
            std::mem::swap(&mut self.color, &mut self.alternate);
            self.clicks += 1;
            self.request_window_redraw();
        }
    }
}

impl ColorTile {
    fn new(elements: &mut Elements) -> Self {
        let inner = elements.insert_element(true, |element_data| ColorTileNode {
            element_data,
            color: rgb(37, 99, 235),
            alternate: rgb(219, 39, 119),
            clicks: 0,
        });
        Self { inner }
    }

    fn fill_color(self, elements: &mut Elements, color: Color) -> Self {
        let tile = elements.get_as_mut::<ColorTileNode>(self.inner);
        tile.color = color;
        tile.request_window_redraw();
        self
    }

    fn click_count(self, elements: &Elements) -> u32 {
        elements.get_as::<ColorTileNode>(self.inner).clicks
    }
}

trait ColorTileEditorExt: Sized {
    fn fill_color(self, color: Color) -> Self;
}

impl ColorTileEditorExt for ElementEditor<'_, ColorTile> {
    fn fill_color(self, color: Color) -> Self {
        self.apply(|tile, elements| {
            tile.fill_color(elements, color);
        })
    }
}

fn main() {
    setup_logging();

    let mut elements = Elements::new();
    let label = Text::new(&mut elements, "Click the custom Element")
        .edit(&mut elements)
        .font_size(20.0)
        .selectable(false)
        .finish();

    let tile = ColorTile::new(&mut elements)
        .edit(&mut elements)
        .fill_color(rgb(37, 99, 235))
        .align_self(AlignSelf::Start)
        .width(px(320))
        .height(px(180))
        .padding_all(px(24))
        .border_radius_all((16.0, 16.0))
        .push(label)
        .finish();

    assert_eq!(tile.click_count(&elements), 0);

    Window::new(&mut elements, "Custom Element")
        .edit(&mut elements)
        .width(pct(100))
        .height(pct(100))
        .push(tile)
        .finish();

    retgui_main(elements, RetGuiOptions::basic("Custom Element"));
}
