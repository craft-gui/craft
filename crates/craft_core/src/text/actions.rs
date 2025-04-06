use crate::text::cached_editor::{hash_text, AttributesRaw, CachedEditor};
use cosmic_text::{Action, Change, Cursor, Edit, Editor, FontSystem, Motion, Shaping};
use std::str::Chars;

use crate::elements::layout_context::MetricsRaw;
use crate::elements::text_input::ImeState;
use crate::geometry::Point;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use clipboard_rs::{Clipboard, ClipboardContext};
use winit::event::Modifiers;

impl CachedEditor<'_> {
    // Motions
    pub(crate) fn move_left(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::Left));
    }

    pub(crate) fn move_right(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::Right));
    }

    pub(crate) fn move_up(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::Up));
    }

    pub(crate) fn move_down(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::Down));
    }

    pub(crate) fn move_to_start(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::Home));
    }

    pub(crate) fn move_to_end(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::End));
    }

    pub(crate) fn move_page_up(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::PageUp));
    }

    pub(crate) fn move_page_down(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Motion(Motion::PageDown));
    }

    // Actions
    pub(crate) fn action_escape(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Escape);
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_enter(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Enter);
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_backspace(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Backspace);
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_delete(&mut self, font_system: &mut FontSystem) {
        self.editor.action(font_system, Action::Delete);
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_insert(&mut self, font_system: &mut FontSystem, chars: Chars) {
        for c in chars {
            self.editor.action(font_system, Action::Insert(c));
        }
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_copy_to_clipboard(&mut self) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        if let Some(selection_text) = self.editor.copy_selection() {
            let clipboard_context = ClipboardContext::new().unwrap();
            clipboard_context.set_text(selection_text).ok();
        }
    }

    pub(crate) fn action_cut_from_clipboard(&mut self) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        if let Some(selection_text) = self.editor.copy_selection() {
            let clipboard_context = ClipboardContext::new().unwrap();
            if self.editor.delete_selection() {
                clipboard_context.set_text(selection_text).ok();
                self.is_dirty = true;
            }
        }
    }

    pub(crate) fn action_paste_from_clipboard(&mut self, font_system: &mut FontSystem) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            let clipboard_context = ClipboardContext::new().unwrap();
            if let Ok(clipboard_text) = clipboard_context.get_text() {
                self.action_insert(font_system, clipboard_text.chars());
                self.is_dirty = true;
            }
        }
    }

    pub(crate) fn action_modifiers_changed(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    pub(crate) fn action_start_drag(&mut self, font_system: &mut FontSystem, location: Point) {
        self.editor.action(
            font_system,
            Action::Click {
                x: location.x as i32,
                y: location.y as i32,
            },
        );
        self.dragging = true;
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_drag(&mut self, font_system: &mut FontSystem, location: Point) {
        self.editor.action(
            font_system,
            Action::Drag {
                x: location.x as i32,
                y: location.y as i32,
            },
        );
        self.dragging = true;
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_end_drag(&mut self) {
        self.dragging = false;
        self.is_dirty = self.editor.redraw();
    }

    pub(crate) fn action_ime_enabled(&mut self) -> ImeState {
        ImeState {
            is_ime_active: true,
            ime_starting_cursor: Some(self.editor.cursor()),
            ime_ending_cursor: None,
        }
    }

    pub(crate) fn action_ime_preedit(
        &mut self,
        ime_state: &ImeState,
        str: &str,
        cursor_info: Option<(usize, usize)>,
    ) -> ImeState {
        let mut result = ime_state.clone();
        let is_cleared = str.is_empty();
        let _hide_cursor = cursor_info.is_none();

        // Deletes all the ime pre-edit text from the editor.
        let delete_ime_pre_edit_text = |editor: &mut Editor| {
            if let Some(previous_ime_ending_cursor) = ime_state.ime_ending_cursor {
                let starting_cursor = ime_state.ime_starting_cursor.unwrap();
                editor.delete_range(starting_cursor, previous_ime_ending_cursor);
            }
        };

        if is_cleared {
            if ime_state.ime_ending_cursor.is_some() {
                self.editor.start_change();
                delete_ime_pre_edit_text(&mut self.editor);
                self.editor.finish_change();
                result.ime_ending_cursor = None;
                self.editor.set_cursor(ime_state.ime_starting_cursor.unwrap());
            }
        } else {
            self.editor.start_change();
            delete_ime_pre_edit_text(&mut self.editor);
            self.editor.insert_at(ime_state.ime_starting_cursor.unwrap(), str, None);
            let change = self.editor.finish_change();
            if let Some(cursor) = maybe_set_cursor_to_last_change_item(&mut self.editor, &change) {
                result.ime_ending_cursor = Some(cursor);
            }
        }

        self.is_dirty = true;

        result
    }

    pub(crate) fn action_ime_commit(&mut self, ime_state: &ImeState, str: &str) -> ImeState {
        let mut result = ime_state.clone();
        result.is_ime_active = false;

        self.editor.start_change();
        // delete_ime_pre_edit_text(&mut cached_editor.editor);
        self.editor.insert_at(result.ime_starting_cursor.unwrap(), str, None);
        let change = self.editor.finish_change();
        if let Some(cursor) = maybe_set_cursor_to_last_change_item(&mut self.editor, &change) {
            result.ime_ending_cursor = Some(cursor);
        }

        self.is_dirty = true;

        result
    }

    pub(crate) fn action_ime_disabled(&mut self) -> ImeState {
        ImeState {
            is_ime_active: false,
            ime_starting_cursor: None,
            ime_ending_cursor: None,
        }
    }

    pub(crate) fn action_set_reload_fonts(&mut self, reload_fonts: bool) {
        if reload_fonts {
            self.is_dirty = true;
        }
    }

    pub(crate) fn action_set_text(&mut self, font_system: &mut FontSystem, text: Option<&String>) {
        let text_hash = text.map(hash_text);
        let text_hash_changed = text_hash.map(|text_hash| text_hash != self.text_hash).unwrap_or(false);

        if text_hash_changed {
            self.is_dirty = true;
            if let Some(text) = text {
                // The user supplied text or attributes changed, and we need to rebuild the buffer.
                self.editor.with_buffer_mut(|buffer| {
                    buffer.set_text(font_system, text, &self.attributes.to_attrs(), Shaping::Advanced);
                });
                self.text_hash = text_hash.unwrap();
            } else {
                // The attributes changed, and we need to rebuild the buffer.
                let buffer_text = self.get_text();
                self.editor.with_buffer_mut(|buffer| {
                    buffer.set_text(font_system, buffer_text.as_str(), &self.attributes.to_attrs(), Shaping::Advanced);
                });
            }
        }
    }

    pub(crate) fn action_set_metrics(&mut self, metrics: MetricsRaw) {
        if metrics != self.metrics {
            self.is_dirty = true;
            self.metrics = metrics;
        }
    }

    pub(crate) fn action_set_attributes(&mut self, attributes: AttributesRaw) {
        if attributes != self.attributes {
            self.is_dirty = true;
            self.attributes = attributes;
        }
    }
}

// Set the cursor to the final cursor location of the last change item.
fn maybe_set_cursor_to_last_change_item(editor: &mut Editor, change: &Option<Change>) -> Option<Cursor> {
    if let Some(change) = change {
        if let Some(change_item) = change.items.last() {
            editor.set_cursor(change_item.end);
            return Some(change_item.end);
        }
    }
    None
}
