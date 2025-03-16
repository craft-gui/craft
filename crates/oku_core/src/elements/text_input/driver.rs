// use accesskit::{Node, NodeId, TreeUpdate};
use crate::elements::text_input::plain_text_editor::PlainEditor;
use parley::{Affinity, Brush, Cursor, FontContext, Layout, LayoutContext, Selection};
use std::fmt::Debug;

/// A short-lived wrapper around [`PlainEditor`].
///
/// This can perform operations which require the editor's layout to
/// be up-to-date by refreshing it as necessary.
pub struct PlainEditorDriver<'a, T>
where
    T: Brush + Clone + Debug + PartialEq + Default,
{
    pub editor: &'a mut PlainEditor<T>,
    pub font_cx: &'a mut FontContext,
    pub layout_cx: &'a mut LayoutContext<T>,
}

impl<T> PlainEditorDriver<'_, T>
where
    T: Brush + Clone + Debug + PartialEq + Default,
{
    // --- MARK: Forced relayout ---
    /// Insert at cursor, or replace selection.
    pub fn insert_or_replace_selection(&mut self, s: &str) {
        assert!(!self.editor.is_composing());

        self.editor.replace_selection(self.font_cx, self.layout_cx, s);
    }

    /// Delete the selection.
    pub fn delete_selection(&mut self) {
        assert!(!self.editor.is_composing());

        self.insert_or_replace_selection("");
    }

    /// Delete the selection or the next cluster (typical ‘delete’ behavior).
    pub fn delete(&mut self) {
        assert!(!self.editor.is_composing());

        if self.editor.selection.is_collapsed() {
            // Upstream cluster range
            if let Some(range) = self.editor.selection.focus().logical_clusters(&self.editor.layout)[1]
                .as_ref()
                .map(|cluster| cluster.text_range())
                .and_then(|range| (!range.is_empty()).then_some(range))
            {
                self.editor.buffer.replace_range(range, "");
                self.update_layout();
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or up to the next word boundary (typical ‘ctrl + delete’ behavior).
    pub fn delete_word(&mut self) {
        assert!(!self.editor.is_composing());

        if self.editor.selection.is_collapsed() {
            let focus = self.editor.selection.focus();
            let start = focus.index();
            let end = focus.next_logical_word(&self.editor.layout).index();
            if self.editor.buffer.get(start..end).is_some() {
                self.editor.buffer.replace_range(start..end, "");
                self.update_layout();
                self.editor
                    .set_selection(Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream).into());
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or the previous cluster (typical ‘backspace’ behavior).
    pub fn backdelete(&mut self) {
        assert!(!self.editor.is_composing());

        if self.editor.selection.is_collapsed() {
            // Upstream cluster
            if let Some(cluster) = self.editor.selection.focus().logical_clusters(&self.editor.layout)[0].clone() {
                let range = cluster.text_range();
                let end = range.end;
                let start = if cluster.is_hard_line_break() || cluster.is_emoji() {
                    // For newline sequences and emoji, delete the previous cluster
                    range.start
                } else {
                    // Otherwise, delete the previous character
                    let Some((start, _)) = self.editor.buffer.get(..end).and_then(|str| str.char_indices().next_back())
                    else {
                        return;
                    };
                    start
                };
                self.editor.buffer.replace_range(start..end, "");
                self.update_layout();
                self.editor
                    .set_selection(Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream).into());
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or back to the previous word boundary (typical ‘ctrl + backspace’ behavior).
    pub fn backdelete_word(&mut self) {
        assert!(!self.editor.is_composing());

        if self.editor.selection.is_collapsed() {
            let focus = self.editor.selection.focus();
            let end = focus.index();
            let start = focus.previous_logical_word(&self.editor.layout).index();
            if self.editor.buffer.get(start..end).is_some() {
                self.editor.buffer.replace_range(start..end, "");
                self.update_layout();
                self.editor
                    .set_selection(Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream).into());
            }
        } else {
            self.delete_selection();
        }
    }

    // --- MARK: IME ---
    /// Set the IME preedit composing text.
    ///
    /// This starts composing. Composing is reset by calling [`clear_compose`](Self::clear_compose).
    /// While composing, it is a logic error to call anything other than
    /// [`Self::set_compose`] or [`Self::clear_compose`].
    ///
    /// The preedit text replaces the current selection if this call starts composing.
    ///
    /// The selection is updated based on `cursor`, which contains the byte offsets relative to the
    /// start of the preedit text. If `cursor` is `None`, the selection and caret are hidden.
    pub fn set_compose(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        debug_assert!(!text.is_empty());
        debug_assert!(cursor.map(|cursor| cursor.1 <= text.len()).unwrap_or(true));

        let start = if let Some(preedit_range) = &self.editor.compose {
            self.editor.buffer.replace_range(preedit_range.clone(), text);
            preedit_range.start
        } else {
            if self.editor.selection.is_collapsed() {
                self.editor.buffer.insert_str(self.editor.selection.text_range().start, text);
            } else {
                self.editor.buffer.replace_range(self.editor.selection.text_range(), text);
            }
            self.editor.selection.text_range().start
        };
        self.editor.compose = Some(start..start + text.len());
        self.editor.show_cursor = cursor.is_some();
        self.update_layout();

        // Select the location indicated by the IME. If `cursor` is none, collapse the selection to
        // a caret at the start of the preedit text. As `self.editor.show_cursor` is `false`, it
        // won't show up.
        let cursor = cursor.unwrap_or((0, 0));
        self.editor.set_selection(Selection::new(
            self.editor.cursor_at(start + cursor.0),
            self.editor.cursor_at(start + cursor.1),
        ));
    }

    /// Stop IME composing.
    ///
    /// This removes the IME preedit text.
    pub fn clear_compose(&mut self) {
        if let Some(preedit_range) = self.editor.compose.take() {
            self.editor.buffer.replace_range(preedit_range.clone(), "");
            self.editor.show_cursor = true;
            self.update_layout();

            self.editor.set_selection(self.editor.cursor_at(preedit_range.start).into());
        }
    }

    // --- MARK: Cursor Movement ---
    /// Move the cursor to the cluster boundary nearest this point in the layout.
    pub fn move_to_point(&mut self, x: f32, y: f32) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(Selection::from_point(&self.editor.layout, x, y));
    }

    /// Move the cursor to a byte index.
    ///
    /// No-op if index is not a char boundary.
    pub fn move_to_byte(&mut self, index: usize) {
        assert!(!self.editor.is_composing());

        if self.editor.buffer.is_char_boundary(index) {
            self.refresh_layout();
            self.editor.set_selection(self.editor.cursor_at(index).into());
        }
    }

    /// Move the cursor to the start of the buffer.
    pub fn move_to_text_start(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(&self.editor.layout, isize::MIN, false));
    }

    /// Move the cursor to the start of the physical line.
    pub fn move_to_line_start(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.line_start(&self.editor.layout, false));
    }

    /// Move the cursor to the end of the buffer.
    pub fn move_to_text_end(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(&self.editor.layout, isize::MAX, false));
    }

    /// Move the cursor to the end of the physical line.
    pub fn move_to_line_end(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.line_end(&self.editor.layout, false));
    }

    /// Move up to the closest physical cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    pub fn move_up(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_line(&self.editor.layout, false));
    }

    /// Move down to the closest physical cluster boundary on the next line, preserving the horizontal position for repeated movements.
    pub fn move_down(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_line(&self.editor.layout, false));
    }

    /// Move to the next cluster left in visual order.
    pub fn move_left(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_visual(&self.editor.layout, false));
    }

    /// Move to the next cluster right in visual order.
    pub fn move_right(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_visual(&self.editor.layout, false));
    }

    /// Move to the next word boundary left.
    pub fn move_word_left(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_visual_word(&self.editor.layout, false));
    }

    /// Move to the next word boundary right.
    pub fn move_word_right(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_visual_word(&self.editor.layout, false));
    }

    /// Select the whole buffer.
    pub fn select_all(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(
            Selection::from_byte_index(&self.editor.layout, 0_usize, Affinity::default()).move_lines(
                &self.editor.layout,
                isize::MAX,
                true,
            ),
        );
    }

    /// Collapse selection into caret.
    pub fn collapse_selection(&mut self) {
        assert!(!self.editor.is_composing());

        self.editor.set_selection(self.editor.selection.collapse());
    }

    /// Move the selection focus point to the start of the buffer.
    pub fn select_to_text_start(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(&self.editor.layout, isize::MIN, true));
    }

    /// Move the selection focus point to the start of the physical line.
    pub fn select_to_line_start(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.line_start(&self.editor.layout, true));
    }

    /// Move the selection focus point to the end of the buffer.
    pub fn select_to_text_end(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(&self.editor.layout, isize::MAX, true));
    }

    /// Move the selection focus point to the end of the physical line.
    pub fn select_to_line_end(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.line_end(&self.editor.layout, true));
    }

    /// Move the selection focus point up to the nearest cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    pub fn select_up(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_line(&self.editor.layout, true));
    }

    /// Move the selection focus point down to the nearest cluster boundary on the next line, preserving the horizontal position for repeated movements.
    pub fn select_down(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_line(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next cluster left in visual order.
    pub fn select_left(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_visual(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next cluster right in visual order.
    pub fn select_right(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_visual(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next word boundary left.
    pub fn select_word_left(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.previous_visual_word(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next word boundary right.
    pub fn select_word_right(&mut self) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.next_visual_word(&self.editor.layout, true));
    }

    /// Select the word at the point.
    pub fn select_word_at_point(&mut self, x: f32, y: f32) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        self.editor.set_selection(Selection::word_from_point(&self.editor.layout, x, y));
    }

    /// Select the physical line at the point.
    pub fn select_line_at_point(&mut self, x: f32, y: f32) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        let line = Selection::line_from_point(&self.editor.layout, x, y);
        self.editor.set_selection(line);
    }

    /// Move the selection focus point to the cluster boundary closest to point.
    pub fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        assert!(!self.editor.is_composing());

        self.refresh_layout();
        // FIXME: This is usually the wrong way to handle selection extension for mouse moves, but not a regression.
        self.editor.set_selection(self.editor.selection.extend_to_point(&self.editor.layout, x, y));
    }

    /// Move the selection focus point to a byte index.
    ///
    /// No-op if index is not a char boundary.
    pub fn extend_selection_to_byte(&mut self, index: usize) {
        assert!(!self.editor.is_composing());

        if self.editor.buffer.is_char_boundary(index) {
            self.refresh_layout();
            self.editor.set_selection(self.editor.selection.extend(self.editor.cursor_at(index)));
        }
    }

    /// Select a range of byte indices.
    ///
    /// No-op if either index is not a char boundary.
    pub fn select_byte_range(&mut self, start: usize, end: usize) {
        assert!(!self.editor.is_composing());

        if self.editor.buffer.is_char_boundary(start) && self.editor.buffer.is_char_boundary(end) {
            self.refresh_layout();
            self.editor.set_selection(Selection::new(self.editor.cursor_at(start), self.editor.cursor_at(end)));
        }
    }

    //#[cfg(feature = "accesskit")]
    /// Select inside the editor based on the selection provided by accesskit.
    // pub fn select_from_accesskit(&mut self, selection: &accesskit::TextSelection) {
    //     assert!(!self.editor.is_composing());
    //
    //     self.refresh_layout();
    //     if let Some(selection) = Selection::from_access_selection(
    //         selection,
    //         &self.editor.layout,
    //         &self.editor.layout_access,
    //     ) {
    //         self.editor.set_selection(selection);
    //     }
    // }

    /// --- MARK: Rendering ---
    //#[cfg(feature = "accesskit")]
    /// Perform an accessibility update.
    // pub fn accessibility(
    //     &mut self,
    //     update: &mut TreeUpdate,
    //     node: &mut Node,
    //     next_node_id: impl FnMut() -> NodeId,
    //     x_offset: f64,
    //     y_offset: f64,
    // ) -> Option<()> {
    //     self.refresh_layout();
    //     self.editor
    //         .accessibility_unchecked(update, node, next_node_id, x_offset, y_offset);
    //     Some(())
    // }

    /// Get the up-to-date layout for this driver.
    pub fn layout(&mut self) -> &Layout<T> {
        self.editor.layout(self.font_cx, self.layout_cx)
    }
    // --- MARK: Internal helpers---
    /// Update the layout if needed.
    pub fn refresh_layout(&mut self) {
        self.editor.refresh_layout(self.font_cx, self.layout_cx);
    }

    /// Update the layout unconditionally.
    fn update_layout(&mut self) {
        self.editor.update_layout(self.font_cx, self.layout_cx);
    }
}
