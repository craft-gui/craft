use std::collections::HashMap;
use std::ops::Range;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use gummy::{AvailableSpace, NodeId};

use parley::{Affinity, ContentWidths, Cursor, Selection};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Point, Rectangle};

use retgui_renderer::text_renderer_data::TextRender;

use ui_events::keyboard::{Key, KeyboardEvent, Modifiers, NamedKey};
use ui_events::pointer::PointerUpdate;

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use winit::dpi;

use crate::app::{GUMMY_TREE, queue_event, request_apply_layout};
use crate::elements::element_data::ElementData;
use crate::elements::text_input::parley_box_to_rect;
use crate::elements::{ElementInternals, TextInputInner};
use crate::events::{Event, EventKind, TextInputChanged};
use crate::layout::layout_context::TextHashKey;
use crate::style::{Style, TextStyleProperty};
use crate::text::parley_editor::{PlainEditor, PlainEditorDriver, PreparedLayout};
use crate::text::text_context::TextContext;
use crate::text::{RangedStyles, text_render_data};

pub struct TextInputState {
    pub(crate) gummy_id: Option<NodeId>,
    origin: Point,

    pub is_active: bool,
    #[allow(dead_code)]
    pub(crate) ime_state: ImeState,
    pub(crate) editor: PlainEditor,

    size_cache: HashMap<TextHashKey, gummy::Size<f32>>,

    // The key used by the interactive editor's presented layout.
    presented_layout_key: Option<TextHashKey>,

    current_render_key: Option<TextHashKey>,
    content_widths: Option<ContentWidths>,
    prepared_layout: Option<PreparedLayout>,

    pub(crate) text_render: Option<TextRender>,
    scale_factor: f64,

    last_click_time: Option<Instant>,
    click_count: u32,
    pointer_down: bool,
    cursor_pos: Point,
    cursor_visible: bool,
    modifiers: Option<Modifiers>,
    start_time: Option<Instant>,
    blink_period: Duration,

    /// True if the node needs laid-out.
    pub is_layout_dirty: bool,
}

impl Clone for TextInputState {
    fn clone(&self) -> Self {
        Self {
            gummy_id: self.gummy_id,
            origin: self.origin,
            is_active: self.is_active,
            ime_state: self.ime_state,
            editor: self.editor.clone(),
            size_cache: self.size_cache.clone(),
            presented_layout_key: self.presented_layout_key,
            current_render_key: self.current_render_key,
            content_widths: self.content_widths,
            prepared_layout: None,
            text_render: self.text_render.clone(),
            scale_factor: self.scale_factor,
            last_click_time: self.last_click_time,
            click_count: self.click_count,
            pointer_down: self.pointer_down,
            cursor_pos: self.cursor_pos,
            cursor_visible: self.cursor_visible,
            modifiers: self.modifiers,
            start_time: self.start_time,
            blink_period: self.blink_period,
            is_layout_dirty: self.is_layout_dirty,
        }
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        let default_style = TextInputInner::get_default_style();
        let mut editor = PlainEditor::new(default_style.get_font_size(), None);
        editor.set_scale(1.0);
        let style_set = editor.edit_styles();
        default_style.add_styles_to_style_set(style_set);
        Self {
            gummy_id: None,
            origin: Default::default(),
            ime_state: ImeState::default(),
            is_active: false,
            editor,
            size_cache: Default::default(),
            presented_layout_key: None,
            current_render_key: None,
            content_widths: None,
            prepared_layout: None,
            text_render: None,
            scale_factor: 1.0,
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: Point::default(),
            cursor_visible: false,
            modifiers: None,
            start_time: None,
            blink_period: Default::default(),
            is_layout_dirty: true,
        }
    }
}

#[derive(Clone, Default, Debug, Copy)]
pub(crate) struct ImeState {
    #[allow(dead_code)]
    pub is_ime_active: bool,
}

impl TextInputState {
    /// Returns the last known pointer position in editor-layout coordinates.
    ///
    /// Editor-layout coordinates are element-local and include the editor scale
    /// and text scroll offset. The point may be outside the text input.
    pub fn cursor_pos(&self) -> Point {
        self.cursor_pos
    }

    pub(crate) fn is_rendered_for(&self, key: TextHashKey) -> bool {
        self.text_render.is_some() && self.current_render_key == Some(key)
    }

    pub(crate) fn needs_final_layout(&self, key: TextHashKey) -> bool {
        self.is_layout_dirty || !self.is_rendered_for(key) || self.presented_layout_key != Some(key)
    }

    /// Sets the pointer position from a pointer-move event.
    ///
    /// The point should be relative to the top left of the window.
    pub fn move_pointer(&mut self, text_context: &mut TextContext, pointer_moved: &PointerUpdate, scroll_y: f64) {
        let prev_pos = self.cursor_pos();
        self.set_pointer_position(pointer_moved.current.logical_point(), scroll_y);
        // macOS seems to generate a spurious move after selecting word?
        if self.is_pointer_down() && prev_pos != self.cursor_pos() && !self.editor.is_composing() {
            self.reset_blink();
            let cursor_pos = self.cursor_pos();
            self.driver(text_context)
                .extend_selection_to_point(cursor_pos.x as f32, cursor_pos.y as f32);
            if let Some(gummy_id) = self.gummy_id {
                request_apply_layout(gummy_id);
            }
        }
    }

    /// Returns a suggested scroll offset (y) to ensure the cursor is visible
    /// within a viewport of the given height.
    pub fn calculate_scroll_to_cursor(&self, viewport_height: f32, current_scroll_y: f32) -> f32 {
        // TODO: Rewrite this function. It is likely incorrect.
        let cursor_rect = if let Some(r) = self.editor.cursor_geometry(1.0) {
            parley_box_to_rect(r)
        } else {
            return current_scroll_y;
        };

        logical_scroll_to_cursor(cursor_rect, self.scale_factor, viewport_height, current_scroll_y)
    }

    /// Sets the text input's content origin in window-logical coordinates.
    ///
    /// The point should be relative to the top left of the window.
    pub fn set_origin(&mut self, origin: &Point) {
        self.origin = *origin;
    }

    fn set_pointer_position(&mut self, pointer: Point, scroll_y: f64) {
        self.cursor_pos = pointer_to_editor_position(pointer, self.origin, scroll_y, self.scale_factor);
    }

    /// Measures an alternate layout constraint without changing the interactive
    /// editor's presented layout, selection, or render data.
    pub fn measure(
        &mut self,
        known_dimensions: gummy::Size<Option<f32>>,
        available_space: gummy::Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> gummy::Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);
        if let Some(size) = self.size_cache.get(&key) {
            return *size;
        }

        self.ensure_prepared_layout(text_context);
        let content_widths = self.content_widths.unwrap();
        let width_constraint = physical_width_constraint(
            known_dimensions.width,
            available_space.width,
            content_widths,
            self.scale_factor,
        );
        let (width, height) = self
            .editor
            .measure_layout(self.prepared_layout.as_mut().unwrap(), width_constraint);

        let size = physical_size_to_logical(width, height, self.scale_factor);
        self.size_cache.insert(key, size);
        size
    }

    fn ensure_prepared_layout(&mut self, text_context: &mut TextContext) {
        if self.prepared_layout.is_none() {
            let prepared = self
                .editor
                .prepare_layout(&mut text_context.font_context, &mut text_context.layout_context);
            self.content_widths = Some(prepared.content_widths());
            self.prepared_layout = Some(prepared);
        }
    }

    pub fn clear_cache(&mut self) {
        self.is_layout_dirty = true;
        self.size_cache.clear();
        self.presented_layout_key = None;
        self.current_render_key = None;
        self.text_render = None;
        self.content_widths = None;
        self.prepared_layout = None;

        if let Some(id) = self.gummy_id {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.mark_dirty(id);
            })
        }
    }

    pub fn finalize_layout(
        &mut self,
        available_space: gummy::Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> gummy::Size<f32> {
        let key = TextHashKey::new(gummy::Size::NONE, available_space);

        if !self.is_layout_dirty
            && self.presented_layout_key == Some(key)
            && let Some(layout) = self.editor.try_layout()
        {
            let size = physical_size_to_logical(layout.width(), layout.height(), self.scale_factor);
            self.size_cache.insert(key, size);
            if !self.is_rendered_for(key) {
                self.current_render_key = Some(key);
                self.text_render = Some(text_render_data::from_editor(layout));
            }
            return size;
        }

        let width_constraint = match available_space.width {
            AvailableSpace::Definite(width) => {
                Some(dpi::PhysicalUnit::from_logical::<f32, f32>(width, self.scale_factor).0)
            }
            AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                self.ensure_prepared_layout(text_context);
                let content_widths = self.content_widths.unwrap();
                physical_width_constraint(None, available_space.width, content_widths, self.scale_factor)
            }
        };

        let (width, height) = if let Some(prepared) = self.prepared_layout.take() {
            self.editor.adopt_prepared_layout(prepared, width_constraint)
        } else {
            self.editor.set_width(width_constraint);
            self.editor
                .refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);
            let layout = self.editor.try_layout().unwrap();
            (layout.width(), layout.height())
        };
        let size = physical_size_to_logical(width, height, self.scale_factor);

        self.size_cache.insert(key, size);
        self.presented_layout_key = Some(key);
        self.current_render_key = Some(key);
        let layout = self.editor.try_layout().unwrap();
        self.text_render = Some(text_render_data::from_editor(layout));
        self.is_layout_dirty = false;
        size
    }

    #[allow(dead_code)]
    pub fn get_cursor_link(&self, cursor_pos: Point, element: &TextInputInner) -> Option<String> {
        if let Some(ranged_styles) = &element.ranged_styles {
            let layout = self.editor.try_layout().unwrap();
            for (range, style) in ranged_styles.styles.iter() {
                if let TextStyleProperty::Link(link) = style {
                    let anchor = Cursor::from_byte_index(layout, range.start, Affinity::Downstream);
                    let focus = Cursor::from_byte_index(layout, range.end, Affinity::Downstream);
                    let selection = Selection::new(anchor, focus);
                    let link_rects = selection.geometry(layout);
                    for link_rect in link_rects {
                        if parley_box_to_rect(link_rect.0).contains(&cursor_pos) {
                            return Some(link.clone());
                        }
                    }
                }
            }
        }
        None
    }

    /// Resets the cursor blink.
    pub fn reset_blink(&mut self) {
        self.start_time = Some(Instant::now());
        // TODO: for real world use, this should be reading from the system settings
        self.blink_period = Duration::from_millis(500);
        self.cursor_visible = true;
    }

    #[allow(dead_code)]
    pub fn disable_blink(&mut self) {
        self.start_time = None;
    }

    #[allow(dead_code)]
    pub fn next_blink_time(&self) -> Option<Instant> {
        self.start_time.map(|start_time| {
            let phase = Instant::now().duration_since(start_time);

            start_time
                + Duration::from_nanos(
                    ((phase.as_nanos() / self.blink_period.as_nanos() + 1) * self.blink_period.as_nanos()) as u64,
                )
        })
    }

    #[allow(dead_code)]
    pub fn cursor_blink(&mut self) {
        self.cursor_visible = self.start_time.is_some_and(|start_time| {
            let elapsed = Instant::now().duration_since(start_time);
            (elapsed.as_millis() / self.blink_period.as_millis()).is_multiple_of(2)
        });
    }

    pub(crate) fn driver<'a>(&'a mut self, text_context: &'a mut TextContext) -> PlainEditorDriver<'a> {
        self.editor
            .driver(&mut text_context.font_context, &mut text_context.layout_context)
    }

    /// Set's the scale factor.
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.editor.set_scale(scale_factor);
        self.clear_cache();
    }

    pub fn set_style(&mut self, style: &Style) {
        let style_set = self.editor.edit_styles();
        style_set.clear();
        style.add_styles_to_style_set(style_set);
        self.editor.set_alignment(match style.get_text_align() {
            crate::style::TextAlign::Start => parley::Alignment::Start,
            crate::style::TextAlign::End => parley::Alignment::End,
            crate::style::TextAlign::Left => parley::Alignment::Left,
            crate::style::TextAlign::Center => parley::Alignment::Center,
            crate::style::TextAlign::Right => parley::Alignment::Right,
            crate::style::TextAlign::Justify => parley::Alignment::Justify,
        });
        self.clear_cache();
    }

    pub fn pointer_down(&mut self, text_context: &mut TextContext, pointer_position: Point, scroll_y: f64) {
        self.set_pointer_position(pointer_position, scroll_y);
        self.cursor_visible = true;
        self.pointer_down = true;
        self.reset_blink();
        if !self.editor.is_composing() {
            let now = Instant::now();
            if let Some(last) = self.last_click_time.take() {
                if now.duration_since(last).as_secs_f64() < 0.25 {
                    self.click_count = (self.click_count + 1) % 4;
                } else {
                    self.click_count = 1;
                }
            } else {
                self.click_count = 1;
            }
            self.last_click_time = Some(now);
            let click_count = self.click_count;
            let cursor_pos = self.cursor_pos;
            let cursor_x = cursor_pos.x as f32;
            let cursor_y = cursor_pos.y as f32;

            if click_count == 1 {
                /*if let Some(_link) = self.get_cursor_link(cursor_pos, element) {
                    // TODO generate event
                    return;
                }*/
            }

            let mut drv = self.driver(text_context);

            match click_count {
                2 => drv.select_word_at_point(cursor_x, cursor_y),
                3 => drv.select_line_at_point(cursor_x, cursor_y),
                _ => drv.move_to_point(cursor_x, cursor_y),
            }
        }
    }

    pub fn pointer_up(&mut self) {
        self.pointer_down = false;
        self.reset_blink();
    }

    pub fn maybe_scroll_to_cursor(&mut self, element_data: &mut ElementData) {
        let height = element_data.layout.computed_box.padding_rectangle_size().height;
        let x = self.calculate_scroll_to_cursor(height, element_data.layout.scroll_state.scroll_y());
        if x < 0.0 {
            return;
        }
        if crate::elements::scrollable::set_scroll_y(&mut element_data.layout, x) {
            element_data.apply_accessibility_scroll_data();
        }
    }

    /// Insert at cursor, or replace selection.
    ///
    /// This requires a relayout.
    pub fn insert_or_replace_selection(&mut self, text_context: &mut TextContext, text: &str) {
        self.driver(text_context).insert_or_replace_selection(text, true);
        self.clear_cache();
    }

    pub fn is_pointer_down(&self) -> bool {
        self.pointer_down
    }

    fn generate_text_changed_event(&self, element_data: &ElementData) {
        let new_event = Event::new(element_data.me.upgrade().unwrap());
        queue_event(
            new_event,
            EventKind::TextInputChanged(TextInputChanged {
                value: self.editor.raw_text().to_string(),
            }),
        );
    }

    pub fn key_press(
        &mut self,
        text_context: &mut TextContext,
        keyboard_event: &KeyboardEvent,
        element_data: &mut ElementData,
    ) {
        // TODO: self.reset_blink();

        self.modifiers = Some(keyboard_event.modifiers);

        const IS_MAC: bool = cfg!(target_os = "macos");

        let (shift, action_mod, word_mod) = self
            .modifiers
            .map(|mods| {
                if IS_MAC {
                    // mac: cmd for actions, alt for words
                    (mods.shift(), mods.meta(), mods.alt())
                } else {
                    // windows/linux: Ctrl for both
                    (mods.shift(), mods.ctrl(), mods.ctrl())
                }
            })
            .unwrap_or_default();

        let mut driver = self.driver(text_context);

        match &keyboard_event.key {
            #[cfg(target_os = "windows")]
            Key::Character(c) if action_mod && c.to_lowercase() == "y" => {
                driver.redo();
                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            Key::Character(c) if action_mod && c.to_lowercase() == "z" => {
                if shift {
                    driver.redo();
                } else {
                    driver.undo();
                }
                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                match c.to_lowercase().as_str() {
                    "c" => copy(&mut driver),
                    "x" => {
                        cut(&mut driver);
                        self.clear_cache();
                        self.generate_text_changed_event(element_data);
                    }
                    "v" => {
                        paste(&mut driver);
                        self.clear_cache();
                        self.generate_text_changed_event(element_data);
                    }
                    _ => (),
                }
            }
            Key::Character(c) if action_mod && matches!(c.to_lowercase().as_str(), "a") => {
                if shift {
                    driver.collapse_selection();
                } else {
                    driver.select_all();
                }
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if IS_MAC && action_mod {
                    // mac: Cmd + Left = Line Start
                    if shift {
                        driver.select_to_line_start();
                    } else {
                        driver.move_to_line_start();
                    }
                } else if word_mod {
                    // windows: ctrl + left | mac: alt + left = word left
                    if shift {
                        driver.select_word_left();
                    } else {
                        driver.move_word_left();
                    }
                } else if shift {
                    driver.select_left();
                } else {
                    driver.move_left();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::ArrowRight) => {
                if IS_MAC && action_mod {
                    // mac: cmd + right = end of line
                    if shift {
                        driver.select_to_line_end();
                    } else {
                        driver.move_to_line_end();
                    }
                } else if word_mod {
                    // windows/linux: ctrl + right | mac: alt + right = word right
                    if shift {
                        driver.select_word_right();
                    } else {
                        driver.move_word_right();
                    }
                } else if shift {
                    driver.select_right();
                } else {
                    driver.move_right();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::ArrowUp) => {
                if IS_MAC && action_mod {
                    // mac: cmd + up = document start
                    if shift {
                        driver.select_to_text_start();
                    } else {
                        driver.move_to_text_start();
                    }
                } else if word_mod {
                    if shift {
                        driver.select_left();
                        driver.select_to_hard_line_start();
                    } else {
                        driver.move_left();
                        driver.move_to_hard_line_start();
                    }
                } else if shift {
                    driver.select_up();
                } else {
                    driver.move_up();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::ArrowDown) => {
                if IS_MAC && action_mod {
                    // mac: cmd + down = end of document
                    if shift {
                        driver.select_to_text_end();
                    } else {
                        driver.move_to_text_end();
                    }
                } else if word_mod {
                    if shift {
                        driver.select_to_hard_line_end();
                        driver.select_right();
                    } else {
                        driver.move_to_hard_line_end();
                        driver.move_right();
                    }
                } else if shift {
                    driver.select_down();
                } else {
                    driver.move_down();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::Home) => {
                if action_mod {
                    if shift {
                        driver.select_to_text_start();
                    } else {
                        driver.move_to_text_start();
                    }
                } else if shift {
                    driver.select_to_line_start();
                } else {
                    driver.move_to_line_start();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::End) => {
                let mut drv = self.driver(text_context);

                if action_mod {
                    if shift {
                        drv.select_to_text_end();
                    } else {
                        drv.move_to_text_end();
                    }
                } else if shift {
                    drv.select_to_line_end();
                } else {
                    drv.move_to_line_end();
                }
                self.maybe_scroll_to_cursor(element_data);
            }
            Key::Named(NamedKey::Delete) => {
                if word_mod {
                    driver.delete_word(true);
                } else {
                    driver.delete(true);
                }
                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            Key::Named(NamedKey::Backspace) => {
                if IS_MAC && action_mod {
                    driver.move_anchor_to_line_start();
                    driver.insert_or_replace_selection("", true);
                } else if word_mod {
                    if IS_MAC {
                        driver.move_anchor_word_left();
                        self.insert_or_replace_selection(text_context, "");
                    } else {
                        driver.backdelete_word(true);
                    }
                } else {
                    driver.backdelete(true);
                }

                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            Key::Named(NamedKey::Enter) => {
                driver.insert_or_replace_selection("\n", true);
                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            Key::Character(character) => {
                driver.insert_or_replace_selection(character, true);
                self.clear_cache();
                self.generate_text_changed_event(element_data);
            }
            _ => (),
        }
    }

    pub fn copy(&mut self, text_context: &mut TextContext) {
        copy(&mut self.driver(text_context));
    }

    pub fn paste(&mut self, text_context: &mut TextContext) {
        paste(&mut self.driver(text_context));
        self.clear_cache();
    }

    pub fn cut(&mut self, text_context: &mut TextContext) {
        cut(&mut self.driver(text_context));
        self.clear_cache();
    }

    pub fn ime_pre_edit(&mut self, text_context: &mut TextContext, text: &str, cursor: &Option<(usize, usize)>) {
        if text.is_empty() {
            self.driver(text_context).clear_compose();
        } else {
            self.driver(text_context).set_compose(text, *cursor);
        }
        self.clear_cache();
    }

    pub fn disable_ime(&mut self, text_context: &mut TextContext) {
        self.driver(text_context).clear_compose();
        self.clear_cache();
    }

    pub fn editor(&self) -> &PlainEditor {
        &self.editor
    }

    pub fn set_text(&mut self, text: &str) {
        self.editor.set_text(text);
        self.clear_cache();
    }

    pub fn set_ranged_styles(&mut self, ranged_styles: RangedStyles) {
        self.editor.set_ranged_styles(ranged_styles);
        self.clear_cache();
    }

    pub fn render_text(&mut self, style: &Style) {
        let backgrounds: Vec<(Range<usize>, Brush)> = self
            .editor()
            .ranged_styles
            .styles
            .iter()
            .filter_map(|(range, style)| {
                if let TextStyleProperty::BackgroundBrush(color) = style {
                    Some((range.clone(), color.clone()))
                } else {
                    None
                }
            })
            .collect();

        let layout = self.editor.try_layout().unwrap();
        let backgrounds: Vec<(Selection, Brush)> = backgrounds
            .iter()
            .map(|(range, color)| {
                (
                    Selection::new(
                        Cursor::from_byte_index(layout, range.start, Affinity::Downstream),
                        Cursor::from_byte_index(layout, range.end, Affinity::Downstream),
                    ),
                    color.clone(),
                )
            })
            .collect();
        let text_renderer = self.text_render.as_mut().unwrap();
        for line in text_renderer.lines.iter_mut() {
            line.backgrounds.clear();
        }
        for (selection, color) in backgrounds.iter() {
            selection.geometry_with(layout, |rect, line| {
                text_renderer.lines[line].backgrounds.push((
                    Rectangle::new(
                        rect.x0 as f32,
                        rect.y0 as f32,
                        rect.width() as f32,
                        rect.height() as f32,
                    ),
                    color.clone(),
                ));
            });
        }

        for line in text_renderer.lines.iter_mut() {
            line.selections.clear();
        }
        self.editor.selection_geometry_with(|rect, line| {
            text_renderer.lines[line]
                .selections
                .push((parley_box_to_rect(rect), style.get_selection_brush()));
        });

        let color = style.get_cursor_brush().unwrap_or(style.get_text_brush());
        text_renderer.cursor = self.editor.cursor_geometry(1.0).map(|r| (parley_box_to_rect(r), color));
    }
}

fn physical_width_constraint(
    known_width: Option<f32>,
    available_width: AvailableSpace,
    content_widths: ContentWidths,
    scale_factor: f64,
) -> Option<f32> {
    if let Some(width) = known_width {
        return Some(dpi::PhysicalUnit::from_logical::<f32, f32>(width, scale_factor).0);
    }

    match available_width {
        AvailableSpace::MinContent => Some(content_widths.min),
        AvailableSpace::MaxContent => Some(content_widths.max),
        AvailableSpace::Definite(width) => Some(dpi::PhysicalUnit::from_logical::<f32, f32>(width, scale_factor).0),
    }
}

fn physical_size_to_logical(width: f32, height: f32, scale_factor: f64) -> gummy::Size<f32> {
    gummy::Size {
        width: dpi::LogicalUnit::from_physical::<f32, f32>(width, scale_factor).0,
        height: dpi::LogicalUnit::from_physical::<f32, f32>(height, scale_factor).0,
    }
}

fn pointer_to_editor_position(pointer: Point, origin: Point, scroll_y: f64, scale_factor: f64) -> Point {
    let local = pointer - origin;
    Point::new(local.x * scale_factor, (local.y + scroll_y) * scale_factor)
}

fn logical_scroll_to_cursor(
    cursor_rect: Rectangle,
    scale_factor: f64,
    viewport_height: f32,
    current_scroll_y: f32,
) -> f32 {
    let scale_factor = scale_factor as f32;
    let margin = 2.0;
    let cursor_top = cursor_rect.top() / scale_factor - margin;
    let cursor_bottom = cursor_rect.bottom() / scale_factor + margin;

    let mut new_scroll = current_scroll_y;

    if cursor_top < current_scroll_y {
        new_scroll = cursor_top;
    } else if cursor_bottom > current_scroll_y + viewport_height {
        new_scroll = cursor_bottom - viewport_height;
    }

    new_scroll.max(0.0)
}

#[cfg(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
))]
fn copy(drv: &mut PlainEditorDriver) {
    use clipboard_rs::{Clipboard, ClipboardContext};
    if let Some(text) = drv.editor.selected_text() {
        let cb = ClipboardContext::new().unwrap();
        cb.set_text(text.to_owned()).ok();
    }
}

#[cfg(not(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
)))]
fn copy(_drv: &mut PlainEditorDriver) {}

#[cfg(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
))]
fn paste(drv: &mut PlainEditorDriver) {
    use clipboard_rs::{Clipboard, ClipboardContext};
    let cb = ClipboardContext::new().unwrap();
    let text = cb.get_text().unwrap_or_default();
    drv.insert_or_replace_selection(&text, true);
}

#[cfg(not(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
)))]
fn paste(_drv: &mut PlainEditorDriver) {}

#[cfg(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
))]
fn cut(drv: &mut PlainEditorDriver) {
    use clipboard_rs::{Clipboard, ClipboardContext};
    if let Some(text) = drv.editor.selected_text() {
        let cb = ClipboardContext::new().unwrap();
        cb.set_text(text.to_owned()).ok();
        drv.delete_selection(true);
    }
}

#[cfg(not(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
)))]
fn cut(_drv: &mut PlainEditorDriver) {}
