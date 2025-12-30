use crate::layout::layout_context::TextHashKey;
use craft_primitives::geometry::{Point, Rectangle};
use craft_renderer::text_renderer_data::TextRender;
use std::collections::HashMap;
use std::ops::Range;
use taffy::{AvailableSpace, NodeId};

use crate::text::parley_editor::{PlainEditor, PlainEditorDriver};

use ui_events::keyboard::{Key, KeyboardEvent, Modifiers, NamedKey};

#[cfg(feature = "accesskit")]
use accesskit::{Node, TreeUpdate};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::app::TAFFY_TREE;
use crate::elements::core::ElementInternals;
use crate::elements::text_input::parley_box_to_rect;
use crate::elements::TextInput;
use crate::style::{Style, TextStyleProperty};
use crate::text::text_context::TextContext;
use crate::text::{text_render_data, RangedStyles};
use parley::{Affinity, ContentWidths, Cursor, Selection};
use peniko::Color;
use ui_events::pointer::PointerUpdate;
use winit::dpi;
use crate::request_layout;

#[derive(Clone)]
pub struct TextInputState {
    origin: Point,

    taffy_node: Option<NodeId>,
    pub is_active: bool,
    #[allow(dead_code)]
    pub(crate) ime_state: ImeState,
    editor: PlainEditor,

    cache: HashMap<TextHashKey, taffy::Size<f32>>,

    // The current key used for laying out the text input.
    current_layout_key: Option<TextHashKey>,

    current_render_key: Option<TextHashKey>,
    content_widths: Option<ContentWidths>,

    // The most recently requested key for laying out the text input.
    pub(crate) last_requested_key: Option<TextHashKey>,
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

impl Default for TextInputState {
    fn default() -> Self {
        let default_style = TextInput::get_default_style();
        let mut editor = PlainEditor::new(default_style.font_size());
        editor.set_scale(1.0f32);
        let style_set = editor.edit_styles();
        default_style.add_styles_to_style_set(style_set);
        Self {
            origin: Default::default(),
            taffy_node: None,
            ime_state: ImeState::default(),
            is_active: false,
            editor,
            cache: Default::default(),
            current_layout_key: None,
            current_render_key: None,
            content_widths: None,
            last_requested_key: None,
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
    /// Returns the last know positon of the cursor relative to the origin.
    ///
    /// The cursor is assumed to start at (0.0, 0.0). The cursor_pos may return points
    /// outside the text input.
    pub fn cursor_pos(&self) -> Point {
        self.cursor_pos
    }

    /// Set the mouse positon.
    ///
    /// The point should be relative to the top left of the window.
    pub fn move_pointer(&mut self, text_context: &mut TextContext, pointer_moved: &PointerUpdate, scroll_y: f64) {
        let prev_pos = self.cursor_pos();
        // NOTE: Cursor position should be relative to the top left of the text box.
        let cursor_pos = pointer_moved.current.logical_point();
        let cursor_pos: Point = (cursor_pos - self.origin).to_point();
        let mut cursor_pos = Point::new(cursor_pos.x * self.scale_factor, cursor_pos.y * self.scale_factor);
        cursor_pos.y += scroll_y;
        self.cursor_pos = cursor_pos;
        // macOS seems to generate a spurious move after selecting word?
        if self.is_pointer_down() && prev_pos != self.cursor_pos() && !self.editor.is_composing() {
            self.reset_blink();
            let cursor_pos = self.cursor_pos();
            self.driver(text_context).extend_selection_to_point(cursor_pos.x as f32, cursor_pos.y as f32);
            request_layout();
        }
    }

    /// Set the origin of the text input state.
    ///
    /// The point should be relative to the top left of the window.
    pub fn set_origin(&mut self, origin: &Point) {
        let diff = *origin - self.origin;
        self.cursor_pos += diff;
        self.origin = *origin;
    }

    pub fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> taffy::Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);

        self.last_requested_key = Some(key);

        self.layout(known_dimensions, available_space, text_context, false)
    }

    pub fn clear_cache(&mut self) {
        self.is_layout_dirty = true;
        self.cache.clear();
        self.current_layout_key = None;
        self.last_requested_key = None;
        self.current_render_key = None;
        self.text_render = None;
        self.content_widths = None;

        if let Some(id) = self.taffy_node {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.mark_dirty(id);
            })
        }
    }

    pub fn layout(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<AvailableSpace>,
        text_context: &mut TextContext,
        last_pass: bool,
    ) -> taffy::Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);

        if let Some(value) = self.cache.get(&key) {
            if last_pass {
                if self.current_layout_key == Some(key) {
                    if self.current_render_key != self.current_layout_key {
                        self.current_render_key = self.current_layout_key;

                        let layout = self.editor.try_layout().unwrap();
                        self.text_render = Some(text_render_data::from_editor(layout));
                    }
                    return *value;
                }
            } else {
                return *value;
            }
        }

        if self.editor.try_layout().is_none() || self.is_layout_dirty || self.content_widths.is_none() {
            self.editor.set_width(None);
            self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);
            self.content_widths = Some(self.editor.try_layout().unwrap().calculate_content_widths());
        }

        let content_widths = self.content_widths.unwrap();
        let width_constraint: Option<f32> = known_dimensions
            .width
            .or(match available_space.width {
                AvailableSpace::MinContent => Some(content_widths.min),
                AvailableSpace::MaxContent => Some(content_widths.max),
                AvailableSpace::Definite(width) => Some(width),
            })
            .map(|width| {
                let width: f32 = dpi::PhysicalUnit::from_logical::<f32, f32>(width, self.scale_factor).0;
                // Taffy may give a min width > max_width.
                // Min-width is preserved in this scenario to ensure text is readable.
                width.clamp(content_widths.min, content_widths.max.max(content_widths.min))
            });

        let _height_constraint: Option<f32> = known_dimensions
            .height
            .or(match available_space.height {
                AvailableSpace::MinContent => None,
                AvailableSpace::MaxContent => None,
                AvailableSpace::Definite(height) => Some(height),
            })
            .map(|height| dpi::PhysicalUnit::from_logical::<f32, f32>(height, self.scale_factor).0);

        self.editor.set_width(width_constraint);
        self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);
        let layout = self.editor.try_layout().unwrap();

        if last_pass {
            self.current_render_key = self.current_layout_key;
            self.text_render = Some(text_render_data::from_editor(layout));
        }

        let logical_width = dpi::LogicalUnit::from_physical::<f32, f32>(layout.width(), self.scale_factor).0;
        let logical_height = dpi::LogicalUnit::from_physical::<f32, f32>(layout.height(), self.scale_factor).0;

        let size = taffy::Size {
            width: logical_width,
            height: logical_height,
        };

        self.cache.insert(key, size);
        self.current_layout_key = Some(key);
        size
    }

    pub fn taffy_node(&mut self, taffy_node: Option<NodeId>) {
        self.taffy_node = taffy_node;
    }

    #[allow(dead_code)]
    pub fn get_cursor_link(&self, cursor_pos: Point, element: &TextInput) -> Option<String> {
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
            (elapsed.as_millis() / self.blink_period.as_millis()) % 2 == 0
        });
    }

    fn driver<'a>(&'a mut self, text_context: &'a mut TextContext) -> PlainEditorDriver<'a> {
        self.editor.driver(&mut text_context.font_context, &mut text_context.layout_context)
    }

    /// Set's the scale factor.
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.editor.set_scale(scale_factor as f32);
        self.clear_cache();
    }

    pub fn pointer_down(&mut self, text_context: &mut TextContext) {
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

    /// Insert at cursor, or replace selection.
    ///
    /// This requires a relayout.
    pub fn insert_or_replace_selection(&mut self, text_context: &mut TextContext, text: &str) {
        self.driver(text_context).insert_or_replace_selection(text);
        self.clear_cache();
    }

    pub fn is_pointer_down(&self) -> bool {
        self.pointer_down
    }

    pub fn key_press(&mut self, text_context: &mut TextContext, keyboard_event: &KeyboardEvent) {
        // TODO: self.reset_blink();

        self.modifiers = Some(keyboard_event.modifiers);

        #[allow(unused)]
        let (shift, action_mod) = self
            .modifiers
            .map(|mods| (mods.shift(), if cfg!(target_os = "macos") { mods.meta() } else { mods.ctrl() }))
            .unwrap_or_default();

        let mut driver = self.driver(text_context);

        match &keyboard_event.key {
            Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                match c.to_lowercase().as_str() {
                    "c" => copy(&mut driver),
                    "x" => {
                        cut(&mut driver);
                        self.clear_cache();
                        //generate_text_changed_event(&mut self.editor);
                    }
                    "v" => {
                        paste(&mut driver);
                        self.clear_cache();
                        //generate_text_changed_event(&mut self.editor);
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
                if action_mod {
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
            }
            Key::Named(NamedKey::ArrowRight) => {
                if action_mod {
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
            }
            Key::Named(NamedKey::ArrowUp) => {
                if shift {
                    driver.select_up();
                } else {
                    driver.move_up();
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                if shift {
                    driver.select_down();
                } else {
                    driver.move_down();
                }
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
            }
            Key::Named(NamedKey::Delete) => {
                if action_mod {
                    driver.delete_word();
                    self.clear_cache();
                } else {
                    driver.delete();
                    self.clear_cache();
                }
                //generate_text_changed_event(&mut self.state.editor);
            }
            Key::Named(NamedKey::Backspace) => {
                if action_mod {
                    driver.backdelete_word();
                    self.clear_cache();
                } else {
                    driver.backdelete();
                    self.clear_cache();
                }
                //generate_text_changed_event(&mut self.state.editor);
            }
            Key::Named(NamedKey::Enter) => {
                driver.insert_or_replace_selection("\n");
                self.clear_cache();
                //generate_text_changed_event(&mut self.state.editor);
            }
            Key::Character(s) => {
                driver.insert_or_replace_selection(s);
                self.clear_cache();
                //generate_text_changed_event(&mut self.state.editor);
            }
            _ => (),
        }
    }

    pub fn copy(&mut self, text_context: &mut TextContext) {
        copy(&mut self.driver(text_context));
    }

    pub fn paste(&mut self, text_context: &mut TextContext) {
        paste(&mut self.driver(text_context));
    }

    pub fn cut(&mut self, text_context: &mut TextContext) {
        cut(&mut self.driver(text_context));
        self.clear_cache();
    }

    pub fn ime_pre_edit(&mut self, text_context: &mut TextContext, text: &String, cursor: &Option<(usize, usize)>) {
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

    pub fn render_text(&mut self, focused: bool, style: &Style) {
        let backgrounds: Vec<(Range<usize>, Color)> = self
            .editor()
            .ranged_styles
            .styles
            .iter()
            .filter_map(|(range, style)| {
                if let TextStyleProperty::BackgroundColor(color) = style {
                    Some((range.clone(), *color))
                } else {
                    None
                }
            })
            .collect();

        let layout = self.editor.try_layout().unwrap();
        let backgrounds: Vec<(Selection, Color)> = backgrounds
            .iter()
            .map(|(range, color)| {
                (
                    Selection::new(
                        Cursor::from_byte_index(layout, range.start, Affinity::Downstream),
                        Cursor::from_byte_index(layout, range.end, Affinity::Downstream),
                    ),
                    *color,
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
                    Rectangle::new(rect.x0 as f32, rect.y0 as f32, rect.width() as f32, rect.height() as f32),
                    *color,
                ));
            });
        }

        for line in text_renderer.lines.iter_mut() {
            line.selections.clear();
        }
        self.editor.selection_geometry_with(|rect, line| {
            text_renderer.lines[line].selections.push((parley_box_to_rect(rect), style.selection_color()));
        });

        if focused {
            let color = style.cursor_color().unwrap_or(style.color());
            text_renderer.cursor = self.editor.cursor_geometry(1.0).map(|r| (parley_box_to_rect(r), color));
        } else {
            text_renderer.cursor = None;
        }
    }

    #[cfg(feature = "accesskit")]
    pub fn try_accessibility(
        &mut self,
        tree: &mut TreeUpdate,
        current_node: &mut Node,
        next_node_id: impl FnMut() -> accesskit::NodeId,
        x_offset: f64,
        y_offset: f64,
    ) {
        self.editor.try_accessibility(tree, current_node, next_node_id, x_offset, y_offset);
    }
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
    drv.insert_or_replace_selection(&text);
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
        drv.delete_selection();
    }
}

#[cfg(not(all(
    any(target_os = "windows", target_os = "macos", target_os = "linux"),
    feature = "clipboard"
)))]
fn cut(_drv: &mut PlainEditorDriver) {}
