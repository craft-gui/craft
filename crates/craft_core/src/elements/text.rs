use crate::components::component::ComponentSpecification;
use crate::components::{Props, UpdateResult};
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::layout_context::{LayoutContext, TaffyTextContext};
use crate::elements::ElementStyles;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::text::text_context::TextContext;
use crate::text::text_render_data::{TextRender, TextRenderGlyph, TextRenderItem, TextRenderItemLine, TextRenderLine};
use crate::{generate_component_methods_no_children, RendererBox};
use parley::{Alignment, AlignmentOptions, FontSettings, FontStack, PositionedLayoutItem, TextStyle};
use peniko::kurbo::{Affine, Line};
use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;
use taffy::{NodeId, Size, TaffyTree};
use tokio::io::AsyncBufReadExt;
use winit::keyboard::Key;
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    element_data: ElementData,
    selectable: bool,
}

pub struct TextState {
    text: String,
    text_render: Option<TextRender>,
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_string(),
            element_data: Default::default(),
            selectable: true,
        }
    }

    pub fn disable_selection(mut self) -> Self {
        self.selectable = false;
        self
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }
}

impl Element for Text {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBoxed> {
        &mut self.element_data.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.element_data.computed_box_transformed;
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer);

        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let fill_color = self.element_data.style.color();

        if let Some(text_render) = state.text_render.as_ref() {
            let text_scroll = None;
            renderer.draw_text(text_render.clone(), content_rectangle, text_scroll, false);
        }
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::Text(TaffyTextContext::new(self.element_data.component_id)),
                )
                .unwrap(),
        );

        self.element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: &CraftMessage, element_state: &mut ElementStateStore) -> UpdateResult {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let content_rect = self.element_data.computed_box.content_rectangle();
        let content_position = content_rect.position();

        // Handle selection.
        if self.selectable {
            match message {
                CraftMessage::PointerButtonEvent(pointer_button) => {
                    let pointer_position = pointer_button.position;
                    let pointer_content_position = pointer_position - content_position;
                    if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    } else {
                    }
                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                CraftMessage::PointerMovedEvent(moved) => UpdateResult::new().prevent_defaults().prevent_propagate(),
                CraftMessage::ModifiersChangedEvent(modifiers_changed) => {
                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                CraftMessage::KeyboardInputEvent(keyboard_input) => {
                    let logical_key = keyboard_input.clone().event.logical_key;
                    let key_state = keyboard_input.event.state;

                    if !key_state.is_pressed() {
                        return UpdateResult::new();
                    }

                    if let Key::Character(text) = logical_key {}

                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                _ => UpdateResult::new(),
            }
        } else {
            UpdateResult::default()
        }
    }

    fn initialize_state(&self, scaling_factor: f64) -> ElementStateStoreItem {
        let text_state = TextState {
            text: self.text.clone(),
            text_render: None,
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_state),
        }
    }

    fn update_state(&self, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();
        
        state.text = self.text.clone();
    }
}

impl Text {
    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}

impl TextState {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        let text_style = TextStyle {
            font_stack: FontStack::Source(Cow::Borrowed("sans-serif")),
            font_size: 16.0,
            font_width: Default::default(),
            font_style: Default::default(),
            font_weight: Default::default(),
            font_variations: FontSettings::List(Cow::Borrowed(&[])),
            font_features: FontSettings::List(Cow::Borrowed(&[])),
            locale: Default::default(),
            brush: Default::default(),
            has_underline: Default::default(),
            underline_offset: Default::default(),
            underline_size: Default::default(),
            underline_brush: Default::default(),
            has_strikethrough: Default::default(),
            strikethrough_offset: Default::default(),
            strikethrough_size: Default::default(),
            strikethrough_brush: Default::default(),
            line_height: 1.2,
            word_spacing: Default::default(),
            letter_spacing: Default::default(),
            word_break: Default::default(),
            overflow_wrap: Default::default(),
        };

        let mut builder = text_context.tree_builder(&text_style);
        builder.push_text(&self.text);

        let (mut layout, _) = builder.build();

        layout.break_all_lines(None);
        layout.align(None, Alignment::Start, AlignmentOptions::default());

        let mut text_render = TextRender { lines: Vec::new() };

        for line in layout.lines() {
            let mut text_render_line = TextRenderLine { items: Vec::new() };

            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                let style = glyph_run.style();
                // We draw underlines under the text, then the strikethrough on top, following:
                // https://drafts.csswg.org/css-text-decor/#painting-order
                let underline: Option<TextRenderItemLine> = if let Some(underline) = &style.underline {
                    let underline_brush = &style.brush;
                    let run_metrics = glyph_run.run().metrics();
                    let offset = match underline.offset {
                        Some(offset) => offset,
                        None => run_metrics.underline_offset,
                    };
                    let width = match underline.size {
                        Some(size) => size,
                        None => run_metrics.underline_size,
                    };
                    // The `offset` is the distance from the baseline to the top of the underline
                    // so we move the line down by half the width
                    // Remember that we are using a y-down coordinate system
                    // If there's a custom width, because this is an underline, we want the custom
                    // width to go down from the default expectation
                    let y = glyph_run.baseline() - offset + width / 2.;

                    let line = Line::new(
                        (glyph_run.offset() as f64, y as f64),
                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                    );
                    Some(TextRenderItemLine { line, width })
                } else {
                    None
                };

                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let synthesis = run.synthesis();
                let glyph_xform = synthesis.skew().map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

                let glyphs = glyph_run.glyphs().map(|glyph| {
                    let gx = x + glyph.x;
                    let gy = y - glyph.y;
                    x += glyph.advance;
                    TextRenderGlyph {
                        id: glyph.id,
                        x: gx,
                        y: gy,
                    }
                });

                let strikethrough = if let Some(strikethrough) = &style.strikethrough {
                    let strikethrough_brush = &style.brush;
                    let run_metrics = glyph_run.run().metrics();
                    let offset = match strikethrough.offset {
                        Some(offset) => offset,
                        None => run_metrics.strikethrough_offset,
                    };
                    let width = match strikethrough.size {
                        Some(size) => size,
                        None => run_metrics.strikethrough_size,
                    };
                    // The `offset` is the distance from the baseline to the *top* of the strikethrough
                    // so we calculate the middle y-position of the strikethrough based on the font's
                    // standard strikethrough width.
                    // Remember that we are using a y-down coordinate system
                    let y = glyph_run.baseline() - offset + run_metrics.strikethrough_size / 2.;

                    let line = Line::new(
                        (glyph_run.offset() as f64, y as f64),
                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                    );
                    Some(TextRenderItemLine { line, width })
                } else {
                    None
                };

                let text_render_item = TextRenderItem {
                    underline,
                    strikethrough,
                    glyph_transform: glyph_xform,
                    font_size,
                    glyphs: glyphs.collect(),
                    font: font.clone(),
                };

                text_render_line.items.push(text_render_item);
            }
            text_render.lines.push(text_render_line);
        }

        self.text_render = Some(text_render);

        let width = layout.width();
        let height = layout.height();

        Size { width, height }
    }
}
