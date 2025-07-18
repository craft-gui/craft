//! A simple plain text editor and related types.
use parley::{
    FontContext, LayoutContext, Rect, StyleProperty, StyleSet,
    layout::{
        Affinity, Alignment, AlignmentOptions, Layout,
        cursor::{Cursor, Selection},
    }
};

extern crate alloc;
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::{
    cmp::PartialEq,
    default::Default,
    fmt::{Debug, Display},
    num::NonZeroUsize,
    ops::Range,
};

#[cfg(feature = "accesskit")]
use parley::layout::LayoutAccessibility;
#[cfg(feature = "accesskit")]
use accesskit::{Node, NodeId, TreeUpdate};
use crate::text::RangedStyles;
use craft_primitives::ColorBrush;

/// Opaque representation of a generation.
///
/// Obtained from [`PlainEditor::generation`].
// Overflow handling: the generations are only compared,
// so wrapping is fine. This could only fail if exactly
// `u32::MAX` generations happen between drawing
// operations. This is implausible and so can be ignored.
#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub struct Generation(u32);

impl Generation {
    /// Make it not what it currently is.
    pub(crate) fn nudge(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// A string which is potentially discontiguous in memory.
///
/// This is returned by [`PlainEditor::text`], as the IME preedit
/// area needs to be efficiently excluded from its return value.
#[derive(Debug, Clone, Copy)]
pub struct SplitString<'source>(pub [&'source str; 2]);

impl<'source> SplitString<'source> {
    
    /// Get the characters of this string.
    #[allow(dead_code)]
    pub fn chars(self) -> impl Iterator<Item = char> + 'source {
        self.into_iter().flat_map(str::chars)
    }
}

impl PartialEq<&'_ str> for SplitString<'_> {
    fn eq(&self, other: &&'_ str) -> bool {
        let [a, b] = self.0;
        let mid = a.len();
        // When our MSRV is 1.80 or above, use split_at_checked instead.
        // is_char_boundary checks bounds
        let (a_1, b_1) = if other.is_char_boundary(mid) {
            other.split_at(mid)
        } else {
            return false;
        };

        a_1 == a && b_1 == b
    }
}
// We intentionally choose not to:
// impl PartialEq<Self> for SplitString<'_> {}
// for simplicity, as the impl wouldn't be useful and is non-trivial

impl Display for SplitString<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b] = self.0;
        write!(f, "{a}{b}")
    }
}

/// Iterate through the source strings.
impl<'source> IntoIterator for SplitString<'source> {
    type Item = &'source str;
    type IntoIter = <[&'source str; 2] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Basic plain text editor with a single style applied to the entire text.
///
/// Internally, this is a wrapper around a string buffer and its corresponding [`Layout`],
/// which is kept up-to-date as needed.
/// This layout is invalidated by a number.
#[derive(Clone)]
pub struct PlainEditor
{
    layout: Layout<ColorBrush>,
    buffer: String,
    default_style: StyleSet<ColorBrush>,
    pub(crate) ranged_styles: RangedStyles,
    #[cfg(feature = "accesskit")]
    layout_access: LayoutAccessibility,
    selection: Selection,
    /// Byte offsets of IME composing preedit text in the text buffer.
    /// `None` if the IME is not currently composing.
    compose: Option<Range<usize>>,
    /// Whether the cursor should be shown. The IME can request to hide the cursor.
    show_cursor: bool,
    width: Option<f32>,
    scale: f32,
    quantize: bool,
    // Simple tracking of when the layout needs to be updated
    // before it can be used for `Selection` calculations or
    // for drawing.
    // Not all operations on `PlainEditor` need to operate on a
    // clean layout, and not all operations trigger a layout.
    layout_dirty: bool,
    // TODO: We could avoid redoing the full text layout if only
    // linebreaking or alignment were changed.
    // linebreak_dirty: bool,
    // alignment_dirty: bool,
    alignment: Alignment,
    generation: Generation,
}

impl PlainEditor
{
    /// Create a new editor, with default font size `font_size`.
    pub fn new(font_size: f32) -> Self {
        Self {
            default_style: StyleSet::new(font_size),
            buffer: Default::default(),
            layout: Default::default(),
            #[cfg(feature = "accesskit")]
            layout_access: Default::default(),
            selection: Default::default(),
            compose: None,
            show_cursor: true,
            width: None,
            scale: 1.0,
            quantize: true,
            layout_dirty: true,
            alignment: Alignment::Start,
            // We don't use the `default` value to start with, as our consumers
            // will choose to use that as their initial value, but will probably need
            // to redraw if they haven't already.
            generation: Generation(1),
            ranged_styles: RangedStyles::new(vec![]),
        }
    }
}

/// A short-lived wrapper around [`PlainEditor`].
///
/// This can perform operations which require the editor's layout to
/// be up-to-date by refreshing it as necessary.
pub struct PlainEditorDriver<'a>
{
    pub editor: &'a mut PlainEditor,
    pub font_cx: &'a mut FontContext,
    pub layout_cx: &'a mut LayoutContext<ColorBrush>,
}

impl PlainEditorDriver<'_>
{
    // --- MARK: Forced relayout ---
    /// Insert at cursor, or replace selection.
    pub fn insert_or_replace_selection(&mut self, s: &str) {
        self.editor
            .replace_selection(self.font_cx, self.layout_cx, s);
    }

    /// Delete the selection.
    pub fn delete_selection(&mut self) {
        self.insert_or_replace_selection("");
    }

    /// Delete the specified numbers of bytes before the selection.
    /// The selection is moved to the left by that number of bytes
    /// but otherwise unchanged.
    ///
    /// The deleted range is clamped to the start of the buffer.
    /// No-op if the start of the range is not a char boundary.
    #[allow(dead_code)]
    pub fn delete_bytes_before_selection(&mut self, len: NonZeroUsize) {
        let old_selection = self.editor.selection;
        let selection_range = old_selection.text_range();
        let range = selection_range.start.saturating_sub(len.get())..selection_range.start;
        if range.is_empty() || !self.editor.buffer.is_char_boundary(range.start) {
            return;
        }
        self.editor.buffer.replace_range(range.clone(), "");
        self.editor
            .update_compose_for_replaced_range(range.clone(), 0);
        self.update_layout();
        let old_anchor = old_selection.anchor();
        let old_focus = old_selection.focus();
        // When doing the equivalent of a backspace on a collapsed selection,
        // always use downstream affinity, as `backdelete` does.
        let (anchor_affinity, focus_affinity) = if old_selection.is_collapsed() {
            (Affinity::Downstream, Affinity::Downstream)
        } else {
            (old_anchor.affinity(), old_focus.affinity())
        };
        self.editor.set_selection(Selection::new(
            Cursor::from_byte_index(
                &self.editor.layout,
                old_anchor.index() - range.len(),
                anchor_affinity,
            ),
            Cursor::from_byte_index(
                &self.editor.layout,
                old_focus.index() - range.len(),
                focus_affinity,
            ),
        ));
    }

    /// Delete the specified numbers of bytes after the selection.
    /// The selection is unchanged.
    ///
    /// The deleted range is clamped to the end of the buffer.
    /// No-op if the end of the range is not a char boundary.
    #[allow(dead_code)]
    pub fn delete_bytes_after_selection(&mut self, len: NonZeroUsize) {
        let selection_range = self.editor.selection.text_range();
        let range = selection_range.end
            ..selection_range
            .end
            .saturating_add(len.get())
            .min(self.editor.buffer.len());
        if range.is_empty() || !self.editor.buffer.is_char_boundary(range.end) {
            return;
        }
        self.editor.buffer.replace_range(range.clone(), "");
        self.editor.update_compose_for_replaced_range(range, 0);
        self.update_layout();
    }

    /// Delete the selection or the next cluster (typical ‘delete’ behavior).
    pub fn delete(&mut self) {
        if self.editor.selection.is_collapsed() {
            // Upstream cluster range
            if let Some(range) = self
                .editor
                .selection
                .focus()
                .logical_clusters(&self.editor.layout)[1]
                .as_ref()
                .map(|cluster| cluster.text_range())
                .and_then(|range| (!range.is_empty()).then_some(range))
            {
                self.editor.buffer.replace_range(range.clone(), "");
                self.editor.update_compose_for_replaced_range(range, 0);
                self.update_layout();
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or up to the next word boundary (typical ‘ctrl + delete’ behavior).
    pub fn delete_word(&mut self) {
        if self.editor.selection.is_collapsed() {
            let focus = self.editor.selection.focus();
            let start = focus.index();
            let end = focus.next_logical_word(&self.editor.layout).index();
            if self.editor.buffer.get(start..end).is_some() {
                self.editor.buffer.replace_range(start..end, "");
                self.editor.update_compose_for_replaced_range(start..end, 0);
                self.update_layout();
                self.editor.set_selection(
                    Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or the previous cluster (typical ‘backspace’ behavior).
    pub fn backdelete(&mut self) {
        if self.editor.selection.is_collapsed() {
            // Upstream cluster
            if let Some(cluster) = self
                .editor
                .selection
                .focus()
                .logical_clusters(&self.editor.layout)[0]
            {
                let range = cluster.text_range();
                let end = range.end;
                let start = if cluster.is_hard_line_break() || cluster.is_emoji() {
                    // For newline sequences and emoji, delete the previous cluster
                    range.start
                } else {
                    // Otherwise, delete the previous character
                    let Some((start, _)) = self
                        .editor
                        .buffer
                        .get(..end)
                        .and_then(|str| str.char_indices().next_back())
                    else {
                        return;
                    };
                    start
                };
                self.editor.buffer.replace_range(start..end, "");
                self.editor.update_compose_for_replaced_range(start..end, 0);
                self.update_layout();
                self.editor.set_selection(
                    Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or back to the previous word boundary (typical ‘ctrl + backspace’ behavior).
    pub fn backdelete_word(&mut self) {
        if self.editor.selection.is_collapsed() {
            let focus = self.editor.selection.focus();
            let end = focus.index();
            let start = focus.previous_logical_word(&self.editor.layout).index();
            if self.editor.buffer.get(start..end).is_some() {
                self.editor.buffer.replace_range(start..end, "");
                self.editor.update_compose_for_replaced_range(start..end, 0);
                self.update_layout();
                self.editor.set_selection(
                    Cursor::from_byte_index(&self.editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    // --- MARK: IME ---
    /// Set the IME preedit composing text.
    ///
    /// This starts composing. Composing is reset by calling [`clear_compose`](Self::clear_compose).
    /// Alternatively, the preedit text can be committed by calling [`finish_compose`](Self::finish_compose).
    ///
    /// The selection and preedit region can be manipulated independently while composing
    /// is active.
    ///
    /// The preedit text replaces the current selection if this call starts composing.
    ///
    /// The selection is updated based on `cursor`, which contains the byte offsets relative to the
    /// start of the preedit text. If `cursor` is `None`, the selection and caret are hidden.
    pub fn set_compose(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        debug_assert!(!text.is_empty());
        debug_assert!(cursor.map(|cursor| cursor.1 <= text.len()).unwrap_or(true));

        let start = if let Some(preedit_range) = &self.editor.compose {
            self.editor
                .buffer
                .replace_range(preedit_range.clone(), text);
            preedit_range.start
        } else {
            if self.editor.selection.is_collapsed() {
                self.editor
                    .buffer
                    .insert_str(self.editor.selection.text_range().start, text);
            } else {
                self.editor
                    .buffer
                    .replace_range(self.editor.selection.text_range(), text);
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

    /// Set the preedit range to a range of byte indices.
    /// This leaves the selection and cursor unchanged.
    ///
    /// No-op if either index is not a char boundary.
    #[allow(dead_code)]
    pub fn set_compose_byte_range(&mut self, start: usize, end: usize) {
        if self.editor.buffer.is_char_boundary(start) && self.editor.buffer.is_char_boundary(end) {
            self.editor.compose = Some(start..end);
            self.update_layout();
        }
    }

    /// Stop IME composing.
    ///
    /// This removes the IME preedit text, shows the cursor if it was hidden,
    /// and moves the cursor to the start of the former preedit region.
    pub fn clear_compose(&mut self) {
        if let Some(preedit_range) = self.editor.compose.take() {
            self.editor.buffer.replace_range(preedit_range.clone(), "");
            self.editor.show_cursor = true;
            self.update_layout();

            self.editor
                .set_selection(self.editor.cursor_at(preedit_range.start).into());
        }
    }

    /// Commit the IME preedit text, if any.
    ///
    /// This doesn't change the selection, but shows the cursor if
    /// it was hidden.
    #[allow(dead_code)]
    pub fn finish_compose(&mut self) {
        if self.editor.compose.take().is_some() {
            self.editor.show_cursor = true;
            self.update_layout();
        }
    }

    // --- MARK: Cursor Movement ---
    /// Move the cursor to the cluster boundary nearest this point in the layout.
    pub fn move_to_point(&mut self, x: f32, y: f32) {
        self.refresh_layout();
        self.editor
            .set_selection(Selection::from_point(&self.editor.layout, x, y));
    }

    /// Move the cursor to a byte index.
    ///
    /// No-op if index is not a char boundary.
    #[allow(dead_code)]
    pub fn move_to_byte(&mut self, index: usize) {
        if self.editor.buffer.is_char_boundary(index) {
            self.refresh_layout();
            self.editor
                .set_selection(self.editor.cursor_at(index).into());
        }
    }

    /// Move the cursor to the start of the buffer.
    pub fn move_to_text_start(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(
            &self.editor.layout,
            isize::MIN,
            false,
        ));
    }

    /// Move the cursor to the start of the physical line.
    pub fn move_to_line_start(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.line_start(&self.editor.layout, false));
    }

    /// Move the cursor to the end of the buffer.
    pub fn move_to_text_end(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(
            &self.editor.layout,
            isize::MAX,
            false,
        ));
    }

    /// Move the cursor to the end of the physical line.
    pub fn move_to_line_end(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.line_end(&self.editor.layout, false));
    }

    /// Move up to the closest physical cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    pub fn move_up(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_line(&self.editor.layout, false),
        );
    }

    /// Move down to the closest physical cluster boundary on the next line, preserving the horizontal position for repeated movements.
    pub fn move_down(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.next_line(&self.editor.layout, false));
    }

    /// Move to the next cluster left in visual order.
    pub fn move_left(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_visual(&self.editor.layout, false),
        );
    }

    /// Move to the next cluster right in visual order.
    pub fn move_right(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .next_visual(&self.editor.layout, false),
        );
    }

    /// Move to the next word boundary left.
    pub fn move_word_left(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_visual_word(&self.editor.layout, false),
        );
    }

    /// Move to the next word boundary right.
    pub fn move_word_right(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .next_visual_word(&self.editor.layout, false),
        );
    }

    /// Select the whole buffer.
    pub fn select_all(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            Selection::from_byte_index(&self.editor.layout, 0_usize, Affinity::default())
                .move_lines(&self.editor.layout, isize::MAX, true),
        );
    }

    /// Collapse selection into caret.
    pub fn collapse_selection(&mut self) {
        self.editor.set_selection(self.editor.selection.collapse());
    }

    /// Move the selection focus point to the start of the buffer.
    pub fn select_to_text_start(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(
            &self.editor.layout,
            isize::MIN,
            true,
        ));
    }

    /// Move the selection focus point to the start of the physical line.
    pub fn select_to_line_start(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.line_start(&self.editor.layout, true));
    }

    /// Move the selection focus point to the end of the buffer.
    pub fn select_to_text_end(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(self.editor.selection.move_lines(
            &self.editor.layout,
            isize::MAX,
            true,
        ));
    }

    /// Move the selection focus point to the end of the physical line.
    pub fn select_to_line_end(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.line_end(&self.editor.layout, true));
    }

    /// Move the selection focus point up to the nearest cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    pub fn select_up(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_line(&self.editor.layout, true),
        );
    }

    /// Move the selection focus point down to the nearest cluster boundary on the next line, preserving the horizontal position for repeated movements.
    pub fn select_down(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.next_line(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next cluster left in visual order.
    pub fn select_left(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_visual(&self.editor.layout, true),
        );
    }

    /// Move the selection focus point to the next cluster right in visual order.
    pub fn select_right(&mut self) {
        self.refresh_layout();
        self.editor
            .set_selection(self.editor.selection.next_visual(&self.editor.layout, true));
    }

    /// Move the selection focus point to the next word boundary left.
    pub fn select_word_left(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .previous_visual_word(&self.editor.layout, true),
        );
    }

    /// Move the selection focus point to the next word boundary right.
    pub fn select_word_right(&mut self) {
        self.refresh_layout();
        self.editor.set_selection(
            self.editor
                .selection
                .next_visual_word(&self.editor.layout, true),
        );
    }

    /// Select the word at the point.
    pub fn select_word_at_point(&mut self, x: f32, y: f32) {
        self.refresh_layout();
        self.editor
            .set_selection(Selection::word_from_point(&self.editor.layout, x, y));
    }

    /// Select the physical line at the point.
    pub fn select_line_at_point(&mut self, x: f32, y: f32) {
        self.refresh_layout();
        let line = Selection::line_from_point(&self.editor.layout, x, y);
        self.editor.set_selection(line);
    }

    /// Move the selection focus point to the cluster boundary closest to point.
    pub fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        self.refresh_layout();
        // FIXME: This is usually the wrong way to handle selection extension for mouse moves, but not a regression.
        self.editor.set_selection(
            self.editor
                .selection
                .extend_to_point(&self.editor.layout, x, y),
        );
    }

    /// Move the selection focus point to a byte index.
    ///
    /// No-op if index is not a char boundary.
    #[allow(dead_code)]
    pub fn extend_selection_to_byte(&mut self, index: usize) {
        if self.editor.buffer.is_char_boundary(index) {
            self.refresh_layout();
            self.editor
                .set_selection(self.editor.selection.extend(self.editor.cursor_at(index)));
        }
    }

    /// Select a range of byte indices.
    ///
    /// No-op if either index is not a char boundary.
    #[allow(dead_code)]
    pub fn select_byte_range(&mut self, start: usize, end: usize) {
        if self.editor.buffer.is_char_boundary(start) && self.editor.buffer.is_char_boundary(end) {
            self.refresh_layout();
            self.editor.set_selection(Selection::new(
                self.editor.cursor_at(start),
                self.editor.cursor_at(end),
            ));
        }
    }

    #[cfg(feature = "accesskit")]
    /// Select inside the editor based on the selection provided by accesskit.
    #[allow(dead_code)]
    pub fn select_from_accesskit(&mut self, selection: &accesskit::TextSelection) {
        self.refresh_layout();
        if let Some(selection) = Selection::from_access_selection(
            selection,
            &self.editor.layout,
            &self.editor.layout_access,
        ) {
            self.editor.set_selection(selection);
        }
    }

    // --- MARK: Rendering ---
    #[cfg(feature = "accesskit")]
    /// Perform an accessibility update.
    #[allow(dead_code)]
    pub fn accessibility(
        &mut self,
        update: &mut TreeUpdate,
        node: &mut Node,
        next_node_id: impl FnMut() -> NodeId,
        x_offset: f64,
        y_offset: f64,
    ) -> Option<()> {
        self.refresh_layout();
        self.editor
            .accessibility_unchecked(update, node, next_node_id, x_offset, y_offset);
        Some(())
    }

    /// Get the up-to-date layout for this driver.
    #[allow(dead_code)]
    pub fn layout(&mut self) -> &Layout<ColorBrush> {
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

impl PlainEditor
{
    /// Run a series of [`PlainEditorDriver`] methods.
    ///
    /// This type is only used to simplify methods which require both
    /// the editor and the provided contexts.
    pub fn driver<'drv>(
        &'drv mut self,
        font_cx: &'drv mut FontContext,
        layout_cx: &'drv mut LayoutContext<ColorBrush>,
    ) -> PlainEditorDriver<'drv> {
        PlainEditorDriver {
            editor: self,
            font_cx,
            layout_cx,
        }
    }

    /// Borrow the current selection. The indices returned by functions
    /// such as [`Selection::text_range`] refer to the raw text buffer,
    /// including the IME preedit region, which can be accessed via
    /// [`PlainEditor::raw_text`].
    #[allow(dead_code)]
    pub fn raw_selection(&self) -> &Selection {
        &self.selection
    }

    /// Borrow the current IME preedit range, if any. These indices refer
    /// to the raw text buffer, which can be accessed via [`PlainEditor::raw_text`].
    #[allow(dead_code)]
    pub fn raw_compose(&self) -> &Option<Range<usize>> {
        &self.compose
    }

    /// If the current selection is not collapsed, returns the text content of
    /// that selection.
    #[allow(dead_code)]
    pub fn selected_text(&self) -> Option<&str> {
        if self.is_composing() {
            return None;
        }
        if !self.selection.is_collapsed() {
            self.buffer.get(self.selection.text_range())
        } else {
            None
        }
    }

    /// Get rectangles, and their corresponding line indices, representing the selected portions of
    /// text.
    #[allow(dead_code)]
    pub fn selection_geometry(&self) -> Vec<(Rect, usize)> {
        // We do not check `self.show_cursor` here, as the IME handling code collapses the
        // selection to a caret in that case.
        self.selection.geometry(&self.layout)
    }

    /// Invoke a callback with each rectangle representing the selected portions of text, and the
    /// indices of the lines to which they belong.
    pub fn selection_geometry_with(&self, f: impl FnMut(Rect, usize)) {
        // We do not check `self.show_cursor` here, as the IME handling code collapses the
        // selection to a caret in that case.
        self.selection.geometry_with(&self.layout, f);
    }

    /// Get a rectangle representing the current caret cursor position.
    ///
    /// There is not always a caret. For example, the IME may have indicated the caret should be
    /// hidden.
    pub fn cursor_geometry(&self, size: f32) -> Option<Rect> {
        self.show_cursor
            .then(|| self.selection.focus().geometry(&self.layout, size))
    }

    /// Get a rectangle bounding the text the user is currently editing.
    ///
    /// This is useful for suggesting an exclusion area to the platform for, e.g., IME candidate
    /// box placement. This bounds the area of the preedit text if present, otherwise it bounds the
    /// selection on the focused line.
    pub fn ime_cursor_area(&self) -> Rect {
        let (area, focus) = if let Some(preedit_range) = &self.compose {
            let selection = Selection::new(
                self.cursor_at(preedit_range.start),
                self.cursor_at(preedit_range.end),
            );

            // Bound the entire preedit text.
            let mut area = None;
            selection.geometry_with(&self.layout, |rect, _| {
                let area = area.get_or_insert(rect);
                *area = area.union(rect);
            });

            (
                area.unwrap_or_else(|| selection.focus().geometry(&self.layout, 0.)),
                selection.focus(),
            )
        } else {
            // Bound the selected parts of the focused line only.
            let focus = self.selection.focus().geometry(&self.layout, 0.);
            let mut area = focus;
            self.selection.geometry_with(&self.layout, |rect, _| {
                if rect.y0 == focus.y0 {
                    area = area.union(rect);
                }
            });

            (area, self.selection.focus())
        };

        // Ensure some context is captured even for tiny or collapsed selections by including a
        // region surrounding the selection. Doing this unconditionally, the IME candidate box
        // usually does not need to jump around when composing starts or the preedit is added to.
        let [upstream, downstream] = focus.logical_clusters(&self.layout);
        let font_size = downstream
            .or(upstream)
            .map(|cluster| cluster.run().font_size())
            .unwrap_or(16.0/*ResolvedStyle::<T>::default().font_size*/);
        // Using 0.6 as an estimate of the average advance
        let inflate = 3. * 0.6 * font_size as f64;
        let editor_width = self.width.map(f64::from).unwrap_or(f64::INFINITY);
        Rect {
            x0: (area.x0 - inflate).max(0.),
            x1: (area.x1 + inflate).min(editor_width),
            y0: area.y0,
            y1: area.y1,
        }
    }

    /// Borrow the text content of the buffer.
    ///
    /// The return value is a `SplitString` because it
    /// excludes the IME preedit region.
    pub fn text(&self) -> SplitString<'_> {
        if let Some(preedit_range) = &self.compose {
            SplitString([
                &self.buffer[..preedit_range.start],
                &self.buffer[preedit_range.end..],
            ])
        } else {
            SplitString([&self.buffer, ""])
        }
    }

    /// Borrow the text content of the buffer, including the IME preedit
    /// region if any.
    ///
    /// Application authors should generally prefer [`text`](Self::text). That method excludes the
    /// IME preedit contents, which are not meaningful for applications to access; the
    /// in-progress IME content is not itself what the user intends to write.
    #[allow(dead_code)]
    pub fn raw_text(&self) -> &str {
        &self.buffer
    }

    /// Get the current `Generation` of the layout, to decide whether to draw.
    ///
    /// You should store the generation the editor was at when you last drew it, and then redraw
    /// when the generation is different (`Generation` is [`PartialEq`], so supports the equality `==` operation).
    #[allow(dead_code)]
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Replace the whole text buffer.
    pub fn set_text(&mut self, is: &str) {
        self.buffer.clear();
        self.buffer.push_str(is);
        self.layout_dirty = true;
        self.compose = None;
    }

    /// Set the width of the layout.
    pub fn set_width(&mut self, width: Option<f32>) {
        self.width = width;
        self.layout_dirty = true;
    }

    /// Set the alignment of the layout.
    #[allow(dead_code)]
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
        self.layout_dirty = true;
    }

    /// Set the scale for the layout.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.layout_dirty = true;
    }

    /// Set whether to quantize the layout coordinates.
    ///
    /// Set `quantize` as `true` to have the layout coordinates aligned to pixel boundaries.
    /// That is the easiest way to avoid blurry text and to receive ready-to-paint layout metrics.
    ///
    /// For advanced rendering use cases you can set `quantize` as `false` and receive
    /// fractional coordinates. This ensures the most accurate results if you want to perform
    /// some post-processing on the coordinates before painting. To avoid blurry text you will
    /// still need to quantize the coordinates just before painting.
    ///
    /// Your should round at least the following:
    /// * Glyph run baseline
    /// * Inline box baseline
    ///   - `box.y = (box.y + box.height).round() - box.height`
    /// * Selection geometry's `y0` & `y1`
    /// * Cursor geometry's `y0` & `y1`
    ///
    /// Keep in mind that for the simple `f32::round` to be effective,
    /// you need to first ensure the coordinates are in physical pixel space.
    #[allow(dead_code)]
    pub fn set_quantize(&mut self, quantize: bool) {
        self.quantize = quantize;
        self.layout_dirty = true;
    }

    /// Modify the styles provided for this editor.
    pub fn edit_styles(&mut self) -> &mut StyleSet<ColorBrush> {
        self.layout_dirty = true;
        &mut self.default_style
    }

    /// Sets the ranged styles provided for this editor.
    pub fn set_ranged_styles(&mut self, ranged_styles: RangedStyles){
        self.layout_dirty = true;
        self.ranged_styles = ranged_styles;
    }


    /// Whether the editor is currently in IME composing mode.
    pub fn is_composing(&self) -> bool {
        self.compose.is_some()
    }

    /// Get the full read-only details from the layout, which will be updated if necessary.
    ///
    /// If the required contexts are not available, then [`refresh_layout`](Self::refresh_layout) can
    /// be called in a scope when they are available, and [`try_layout`](Self::try_layout) can
    /// be used instead.
    pub fn layout(
        &mut self,
        font_cx: &mut FontContext,
        layout_cx: &mut LayoutContext<ColorBrush>,
    ) -> &Layout<ColorBrush> {
        self.refresh_layout(font_cx, layout_cx);
        &self.layout
    }

    // --- MARK: Raw APIs ---
    /// Get the full read-only details from the layout, if valid.
    ///
    /// Returns `None` if the layout is not up-to-date.
    /// You can call [`refresh_layout`](Self::refresh_layout) before using this method,
    /// to ensure that the layout is up-to-date.
    ///
    /// The [`layout`](Self::layout) method should generally be preferred.
    pub fn try_layout(&self) -> Option<&Layout<ColorBrush>> {
        if self.layout_dirty {
            None
        } else {
            Some(&self.layout)
        }
    }

    #[cfg(feature = "accesskit")]
    #[inline]
    /// Perform an accessibility update if the layout is valid.
    ///
    /// Returns `None` if the layout is not up-to-date.
    /// You can call [`refresh_layout`](Self::refresh_layout) before using this method,
    /// to ensure that the layout is up-to-date.
    /// The [`accessibility`](PlainEditorDriver::accessibility) method on the driver type
    /// should be preferred if the contexts are available, which will do this automatically.
    pub fn try_accessibility(
        &mut self,
        update: &mut TreeUpdate,
        node: &mut Node,
        next_node_id: impl FnMut() -> NodeId,
        x_offset: f64,
        y_offset: f64,
    ) -> Option<()> {
        if self.layout_dirty {
            return None;
        }
        self.accessibility_unchecked(update, node, next_node_id, x_offset, y_offset);
        Some(())
    }

    /// Update the layout if it is dirty.
    ///
    /// This should only be used alongside [`try_layout`](Self::try_layout)
    /// or [`try_accessibility`](Self::try_accessibility), if those will be
    /// called in a scope where the contexts are not available.
    pub fn refresh_layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<ColorBrush>) {
        if self.layout_dirty {
            self.update_layout(font_cx, layout_cx);
        }
    }

    // --- MARK: Internal Helpers ---
    /// Make a cursor at a given byte index.
    fn cursor_at(&self, index: usize) -> Cursor {
        // TODO: Do we need to be non-dirty?
        // FIXME: `Selection` should make this easier
        if index >= self.buffer.len() {
            Cursor::from_byte_index(&self.layout, self.buffer.len(), Affinity::Upstream)
        } else {
            Cursor::from_byte_index(&self.layout, index, Affinity::Downstream)
        }
    }

    fn update_compose_for_replaced_range(&mut self, old_range: Range<usize>, new_len: usize) {
        if new_len == old_range.len() {
            return;
        }
        let Some(compose) = &mut self.compose else {
            return;
        };
        if compose.end <= old_range.start {
            return;
        }
        if compose.start >= old_range.end {
            if new_len > old_range.len() {
                let diff = new_len - old_range.len();
                *compose = compose.start + diff..compose.end + diff;
            } else {
                let diff = old_range.len() - new_len;
                *compose = compose.start - diff..compose.end - diff;
            }
            return;
        }
        if new_len < old_range.len() {
            if compose.start >= (old_range.start + new_len) {
                self.compose = None;
                return;
            }
            compose.end = compose.end.min(old_range.start + new_len);
        }
    }

    fn replace_selection(
        &mut self,
        font_cx: &mut FontContext,
        layout_cx: &mut LayoutContext<ColorBrush>,
        s: &str,
    ) {
        let range = self.selection.text_range();
        let start = range.start;
        if self.selection.is_collapsed() {
            self.buffer.insert_str(start, s);
        } else {
            self.buffer.replace_range(range.clone(), s);
        }
        self.update_compose_for_replaced_range(range, s.len());

        self.update_layout(font_cx, layout_cx);
        let new_index = start.saturating_add(s.len());
        let affinity = if s.ends_with("\n") {
            Affinity::Downstream
        } else {
            Affinity::Upstream
        };
        self.set_selection(Cursor::from_byte_index(&self.layout, new_index, affinity).into());
    }

    /// Update the selection, and nudge the `Generation` if something other than `h_pos` changed.
    fn set_selection(&mut self, new_sel: Selection) {
        if new_sel.focus() != self.selection.focus() || new_sel.anchor() != self.selection.anchor()
        {
            self.generation.nudge();
        }
        
        self.selection = new_sel;
    }
    /// Update the layout.
    fn update_layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<ColorBrush>) {
        let mut builder =
            layout_cx.ranged_builder(font_cx, &self.buffer, self.scale, self.quantize);
        for prop in self.default_style.inner().values() {
            builder.push_default(prop.to_owned());
        }

        for (range, style) in &self.ranged_styles.styles {
            if let Some(parley_style) = style.to_parley_style_property() {
                builder.push(parley_style, range.clone());
            }
        }

        if let Some(preedit_range) = &self.compose {
            builder.push(StyleProperty::Underline(true), preedit_range.clone());
        }
        self.layout = builder.build(&self.buffer);
        self.layout.break_all_lines(self.width);
        self.layout
            .align(self.width, self.alignment, AlignmentOptions::default());
        self.selection = self.selection.refresh(&self.layout);
        self.layout_dirty = false;
        self.generation.nudge();
    }

    #[cfg(feature = "accesskit")]
    /// Perform an accessibility update, assuming that the layout is valid.
    ///
    /// The wrapper [`accessibility`](PlainEditorDriver::accessibility) on the driver type should
    /// be preferred.
    ///
    /// You should always call [`refresh_layout`](Self::refresh_layout) before using this method,
    /// with no other modifying method calls in between.
    fn accessibility_unchecked(
        &mut self,
        update: &mut TreeUpdate,
        node: &mut Node,
        next_node_id: impl FnMut() -> NodeId,
        x_offset: f64,
        y_offset: f64,
    ) {
        self.layout_access.build_nodes(
            &self.buffer,
            &self.layout,
            update,
            node,
            next_node_id,
            x_offset,
            y_offset,
        );
        if self.show_cursor {
            if let Some(selection) = self
                .selection
                .to_access_selection(&self.layout, &self.layout_access)
            {
                node.set_text_selection(selection);
            }
        } else {
            node.clear_text_selection();
        }
        node.add_action(accesskit::Action::SetTextSelection);
    }
}
