use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::ops::Range;
use kurbo::Rect;
use parley::{Affinity, Alignment, AlignmentOptions, Brush, Cursor, FontContext, Layout, LayoutContext, Selection, StyleProperty, StyleSet};
use crate::text::parley_editor::{Generation, SplitString};

#[cfg(feature = "accesskit")]
use parley::layout::LayoutAccessibility;
#[cfg(feature = "accesskit")]
use accesskit::{Node, NodeId, TreeUpdate};

#[derive(Clone)]
struct EditorData<T> where
    T: Brush + Clone + Debug + PartialEq + Default, {
    layout: Layout<T>,
    buffer: String,
    default_style: StyleSet<T>,
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

trait Editor<T> where T: Brush + Clone + Debug + PartialEq + Default {
    fn editor_data(&self) -> &EditorData<T>;
    fn editor_data_mut(&mut self) -> &mut EditorData<T>;
    
    /// Insert at cursor, or replace selection.
    fn insert_or_replace_selection(&mut self, s: &str) {
        self.editor_data_mut().replace_selection(s);
    }

    /// Delete the selection.
    fn delete_selection(&mut self) {
        self.insert_or_replace_selection("");
    }

    /// Delete the specified numbers of bytes before the selection.
    /// The selection is moved to the left by that number of bytes
    /// but otherwise unchanged.
    ///
    /// The deleted range is clamped to the start of the buffer.
    /// No-op if the start of the range is not a char boundary.
    fn delete_bytes_before_selection(&mut self, len: NonZeroUsize) {
        let editor = self.editor_data_mut();
        let old_selection = editor.selection;
        let selection_range = old_selection.text_range();
        let range = selection_range.start.saturating_sub(len.get())..selection_range.start;
        if range.is_empty() || !editor.buffer.is_char_boundary(range.start) {
            return;
        }
        editor.buffer.replace_range(range.clone(), "");
        editor.update_compose_for_replaced_range(range.clone(), 0);
        
        // self.update_layout();
        editor.layout_dirty = true;
        
        let old_anchor = old_selection.anchor();
        let old_focus = old_selection.focus();
        // When doing the equivalent of a backspace on a collapsed selection,
        // always use downstream affinity, as `backdelete` does.
        let (anchor_affinity, focus_affinity) = if old_selection.is_collapsed() {
            (Affinity::Downstream, Affinity::Downstream)
        } else {
            (old_anchor.affinity(), old_focus.affinity())
        };
        editor.set_selection(Selection::new(
            Cursor::from_byte_index(
                &editor.layout,
                old_anchor.index() - range.len(),
                anchor_affinity,
            ),
            Cursor::from_byte_index(
                &editor.layout,
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
    fn delete_bytes_after_selection(&mut self, len: NonZeroUsize) {
        let editor = self.editor_data_mut();
        
        let selection_range = editor.selection.text_range();
        let range = selection_range.end
            ..selection_range
            .end
            .saturating_add(len.get())
            .min(editor.buffer.len());
        if range.is_empty() || !editor.buffer.is_char_boundary(range.end) {
            return;
        }
        editor.buffer.replace_range(range.clone(), "");
        editor.update_compose_for_replaced_range(range, 0);
        
        // self.update_layout();
        editor.layout_dirty = true;
    }

    /// Delete the selection or the next cluster (typical ‘delete’ behavior).
    fn delete(&mut self) {
        let editor = self.editor_data_mut();
        
        if editor.selection.is_collapsed() {
            // Upstream cluster range
            if let Some(range) = editor
                .selection
                .focus()
                .logical_clusters(&editor.layout)[1]
                .as_ref()
                .map(|cluster| cluster.text_range())
                .and_then(|range| (!range.is_empty()).then_some(range))
            {
                editor.buffer.replace_range(range.clone(), "");
                editor.update_compose_for_replaced_range(range, 0);
                
                // update_layout();
                editor.layout_dirty = true;
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or up to the next word boundary (typical ‘ctrl + delete’ behavior).
    fn delete_word(&mut self) {
        let editor = self.editor_data_mut();
        
        if editor.selection.is_collapsed() {
            let focus = editor.selection.focus();
            let start = focus.index();
            let end = focus.next_logical_word(&editor.layout).index();
            if editor.buffer.get(start..end).is_some() {
                editor.buffer.replace_range(start..end, "");
                editor.update_compose_for_replaced_range(start..end, 0);
                
                // self.update_layout();
                editor.layout_dirty = true;
                
                editor.set_selection(
                    Cursor::from_byte_index(&editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or the previous cluster (typical ‘backspace’ behavior).
    fn backdelete(&mut self) {
        let editor = self.editor_data_mut();
        
        if editor.selection.is_collapsed() {
            // Upstream cluster
            if let Some(cluster) = editor
                .selection
                .focus()
                .logical_clusters(&editor.layout)[0]
                .clone()
            {
                let range = cluster.text_range();
                let end = range.end;
                let start = if cluster.is_hard_line_break() || cluster.is_emoji() {
                    // For newline sequences and emoji, delete the previous cluster
                    range.start
                } else {
                    // Otherwise, delete the previous character
                    let Some((start, _)) = editor
                        .buffer
                        .get(..end)
                        .and_then(|str| str.char_indices().next_back())
                    else {
                        return;
                    };
                    start
                };
                editor.buffer.replace_range(start..end, "");
                editor.update_compose_for_replaced_range(start..end, 0);

                // self.update_layout();
                editor.layout_dirty = true;
                
                editor.set_selection(
                    Cursor::from_byte_index(&editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or back to the previous word boundary (typical ‘ctrl + backspace’ behavior).
    fn backdelete_word(&mut self) {
        let editor = self.editor_data_mut();
        
        if editor.selection.is_collapsed() {
            let focus = editor.selection.focus();
            let end = focus.index();
            let start = focus.previous_logical_word(&editor.layout).index();
            if editor.buffer.get(start..end).is_some() {
                editor.buffer.replace_range(start..end, "");
                editor.update_compose_for_replaced_range(start..end, 0);
                
                // self.update_layout();
                editor.layout_dirty = true;
                
                editor.set_selection(
                    Cursor::from_byte_index(&editor.layout, start, Affinity::Downstream)
                        .into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

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
    fn set_compose(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        debug_assert!(!text.is_empty());
        debug_assert!(cursor.map(|cursor| cursor.1 <= text.len()).unwrap_or(true));

        let editor = self.editor_data_mut();
        
        let start = if let Some(preedit_range) = &editor.compose {
            editor
                .buffer
                .replace_range(preedit_range.clone(), text);
            preedit_range.start
        } else {
            if editor.selection.is_collapsed() {
               editor
                    .buffer
                    .insert_str(editor.selection.text_range().start, text);
            } else {
                editor
                    .buffer
                    .replace_range(editor.selection.text_range(), text);
            }
            editor.selection.text_range().start
        };
        editor.compose = Some(start..start + text.len());
        editor.show_cursor = cursor.is_some();
        
        // self.update_layout();
        editor.layout_dirty = true;
        
        // Select the location indicated by the IME. If `cursor` is none, collapse the selection to
        // a caret at the start of the preedit text. As `self.editor.show_cursor` is `false`, it
        // won't show up.
        let cursor = cursor.unwrap_or((0, 0));
        editor.set_selection(Selection::new(
            editor.cursor_at(start + cursor.0),
            editor.cursor_at(start + cursor.1),
        ));
    }

    /// Set the preedit range to a range of byte indices.
    /// This leaves the selection and cursor unchanged.
    ///
    /// No-op if either index is not a char boundary.
    fn set_compose_byte_range(&mut self, start: usize, end: usize) {
        let editor = self.editor_data_mut();
        if editor.buffer.is_char_boundary(start) && editor.buffer.is_char_boundary(end) {
            editor.compose = Some(start..end);
            
            // self.update_layout();
            editor.layout_dirty = true;
        }
    }

    /// Stop IME composing.
    ///
    /// This removes the IME preedit text, shows the cursor if it was hidden,
    /// and moves the cursor to the start of the former preedit region.
    fn clear_compose(&mut self) {
        let editor = self.editor_data_mut();
        
        if let Some(preedit_range) = editor.compose.take() {
            editor.buffer.replace_range(preedit_range.clone(), "");
            editor.show_cursor = true;
            
            // self.update_layout();
            editor.layout_dirty = true;
            
            editor
                .set_selection(editor.cursor_at(preedit_range.start).into());
        }
    }

    /// Commit the IME preedit text, if any.
    ///
    /// This doesn't change the selection, but shows the cursor if
    /// it was hidden.
   fn finish_compose(&mut self) {
        let editor = self.editor_data_mut();
        
        if editor.compose.take().is_some() {
            editor.show_cursor = true;
            
            // self.update_layout();
            editor.layout_dirty = true;
        }
    }

    // --- MARK: Cursor Movement ---
    /// Move the cursor to the cluster boundary nearest this point in the layout.
    fn move_to_point(&mut self, x: f32, y: f32) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor
            .set_selection(Selection::from_point(&editor.layout, x, y));
    }

    /// Move the cursor to a byte index.
    ///
    /// No-op if index is not a char boundary.
    fn move_to_byte(&mut self, index: usize) {
        let editor = self.editor_data_mut();
        
        if editor.buffer.is_char_boundary(index) {
            // self.refresh_layout();
            editor
                .set_selection(editor.cursor_at(index).into());
        }
    }

    /// Move the cursor to the start of the buffer.
    fn move_to_text_start(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(editor.selection.move_lines(
            &editor.layout,
            isize::MIN,
            false,
        ));
    }

    /// Move the cursor to the start of the physical line.
    fn move_to_line_start(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor
            .set_selection(editor.selection.line_start(&editor.layout, false));
    }

    /// Move the cursor to the end of the buffer.
    fn move_to_text_end(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(editor.selection.move_lines(
            &editor.layout,
            isize::MAX,
            false,
        ));
    }

    /// Move the cursor to the end of the physical line.
    fn move_to_line_end(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor
            .set_selection(editor.selection.line_end(&editor.layout, false));
    }

    /// Move up to the closest physical cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    fn move_up(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(
            editor
                .selection
                .previous_line(&editor.layout, false),
        );
    }

    /// Move down to the closest physical cluster boundary on the next line, preserving the horizontal position for repeated movements.
    fn move_down(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor
            .set_selection(editor.selection.next_line(&editor.layout, false));
    }

    /// Move to the next cluster left in visual order.
    fn move_left(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(
            editor
                .selection
                .previous_visual(&editor.layout, false),
        );
    }

    /// Move to the next cluster right in visual order.
    fn move_right(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(
            editor
                .selection
                .next_visual(&editor.layout, false),
        );
    }

    /// Move to the next word boundary left.
    fn move_word_left(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor.set_selection(
            editor
                .selection
                .previous_visual_word(&editor.layout, false),
        );
    }

    /// Move to the next word boundary right.
    fn move_word_right(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            editor
                .selection
                .next_visual_word(&editor.layout, false),
        );
    }

    /// Select the whole buffer.
    fn select_all(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            Selection::from_byte_index(&editor.layout, 0_usize, Affinity::default())
                .move_lines(&editor.layout, isize::MAX, true),
        );
    }

    /// Collapse selection into caret.
    fn collapse_selection(&mut self) {
        let editor = self.editor_data_mut();
        editor.set_selection(editor.selection.collapse());
    }

    /// Move the selection focus point to the start of the buffer.
    fn select_to_text_start(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(editor.selection.move_lines(
            &editor.layout,
            isize::MIN,
            true,
        ));
    }

    /// Move the selection focus point to the start of the physical line.
    fn select_to_line_start(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        editor
            .set_selection(editor.selection.line_start(&editor.layout, true));
    }

    /// Move the selection focus point to the end of the buffer.
    fn select_to_text_end(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(editor.selection.move_lines(
            &editor.layout,
            isize::MAX,
            true,
        ));
    }

    /// Move the selection focus point to the end of the physical line.
    fn select_to_line_end(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor
            .set_selection(editor.selection.line_end(&editor.layout, true));
    }

    /// Move the selection focus point up to the nearest cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    fn select_up(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            editor
                .selection
                .previous_line(&editor.layout, true),
        );
    }

    /// Move the selection focus point down to the nearest cluster boundary on the next line, preserving the horizontal position for repeated movements.
    fn select_down(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor
            .set_selection(editor.selection.next_line(&editor.layout, true));
    }

    /// Move the selection focus point to the next cluster left in visual order.
    fn select_left(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            editor
                .selection
                .previous_visual(&editor.layout, true),
        );
    }

    /// Move the selection focus point to the next cluster right in visual order.
    fn select_right(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor
            .set_selection(editor.selection.next_visual(&editor.layout, true));
    }

    /// Move the selection focus point to the next word boundary left.
    fn select_word_left(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            editor
                .selection
                .previous_visual_word(&editor.layout, true),
        );
    }

    /// Move the selection focus point to the next word boundary right.
    fn select_word_right(&mut self) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        editor.set_selection(
            editor
                .selection
                .next_visual_word(&editor.layout, true),
        );
    }

    /// Select the word at the point.
    fn select_word_at_point(&mut self, x: f32, y: f32) {
        // self.refresh_layout();
        
        let editor = self.editor_data_mut();
        editor
            .set_selection(Selection::word_from_point(&editor.layout, x, y));
    }

    /// Select the physical line at the point.
    fn select_line_at_point(&mut self, x: f32, y: f32) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        let line = Selection::line_from_point(&editor.layout, x, y);
        editor.set_selection(line);
    }

    /// Move the selection focus point to the cluster boundary closest to point.
    fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        // FIXME: This is usually the wrong way to handle selection extension for mouse moves, but not a regression.
        editor.set_selection(
            editor
                .selection
                .extend_to_point(&editor.layout, x, y),
        );
    }

    /// Move the selection focus point to a byte index.
    ///
    /// No-op if index is not a char boundary.
    fn extend_selection_to_byte(&mut self, index: usize) {
        let editor = self.editor_data_mut();
        
        if editor.buffer.is_char_boundary(index) {
            
            // self.refresh_layout();
            
            editor
                .set_selection(editor.selection.extend(editor.cursor_at(index)));
        }
    }

    /// Select a range of byte indices.
    ///
    /// No-op if either index is not a char boundary.
    fn select_byte_range(&mut self, start: usize, end: usize) {
        let editor = self.editor_data_mut();
        
        if editor.buffer.is_char_boundary(start) && editor.buffer.is_char_boundary(end) {
            
            // self.refresh_layout();
            
            editor.set_selection(Selection::new(
                editor.cursor_at(start),
                editor.cursor_at(end),
            ));
        }
    }

    #[cfg(feature = "accesskit")]
    /// Select inside the editor based on the selection provided by accesskit.
    fn select_from_accesskit(&mut self, selection: &accesskit::TextSelection) {
        let editor = self.editor_data_mut();
        
        // self.refresh_layout();
        
        if let Some(selection) = Selection::from_access_selection(
            selection,
            &editor.layout,
            &editor.layout_access,
        ) {
            editor.set_selection(selection);
        }
    }
}

impl<T> EditorData<T>
where
    T: Brush + Clone + Debug + PartialEq + Default,
{
    /// Borrow the current selection. The indices returned by functions
    /// such as [`Selection::text_range`] refer to the raw text buffer,
    /// including the IME preedit region, which can be accessed via
    /// [`PlainEditor::raw_text`].
    pub fn raw_selection(&self) -> &Selection {
        &self.selection
    }

    /// Borrow the current IME preedit range, if any. These indices refer
    /// to the raw text buffer, which can be accessed via [`PlainEditor::raw_text`].
    pub fn raw_compose(&self) -> &Option<Range<usize>> {
        &self.compose
    }

    /// If the current selection is not collapsed, returns the text content of
    /// that selection.
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
    pub fn raw_text(&self) -> &str {
        &self.buffer
    }

    /// Get the current `Generation` of the layout, to decide whether to draw.
    ///
    /// You should store the generation the editor was at when you last drew it, and then redraw
    /// when the generation is different (`Generation` is [`PartialEq`], so supports the equality `==` operation).
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
    pub fn set_quantize(&mut self, quantize: bool) {
        self.quantize = quantize;
        self.layout_dirty = true;
    }

    /// Modify the styles provided for this editor.
    pub fn edit_styles(&mut self) -> &mut StyleSet<T> {
        self.layout_dirty = true;
        &mut self.default_style
    }

    /// Whether the editor is currently in IME composing mode.
    pub fn is_composing(&self) -> bool {
        self.compose.is_some()
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

    fn replace_selection(&mut self, s: &str) {
        let range = self.selection.text_range();
        let start = range.start;
        if self.selection.is_collapsed() {
            self.buffer.insert_str(start, s);
        } else {
            self.buffer.replace_range(range.clone(), s);
        }
    
        self.update_compose_for_replaced_range(range, s.len());

        // self.update_layout(font_cx, layout_cx);
        self.layout_dirty = true;
        
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

        // This debug code is quite useful when diagnosing selection problems.
        #[cfg(feature = "std")]
        #[allow(clippy::print_stderr)] // reason = "unreachable debug code"
        if false {
            let focus = new_sel.focus();
            let cluster = focus.logical_clusters(&self.layout);
            let dbg = (
                cluster[0].as_ref().map(|c| &self.buffer[c.text_range()]),
                focus.index(),
                focus.affinity(),
                cluster[1].as_ref().map(|c| &self.buffer[c.text_range()]),
            );
            eprint!("{dbg:?}");
            let cluster = focus.visual_clusters(&self.layout);
            let dbg = (
                cluster[0].as_ref().map(|c| &self.buffer[c.text_range()]),
                cluster[0]
                    .as_ref()
                    .map(|c| if c.is_word_boundary() { " W" } else { "" })
                    .unwrap_or_default(),
                focus.index(),
                focus.affinity(),
                cluster[1].as_ref().map(|c| &self.buffer[c.text_range()]),
                cluster[1]
                    .as_ref()
                    .map(|c| if c.is_word_boundary() { " W" } else { "" })
                    .unwrap_or_default(),
            );
            eprintln!(" | visual: {dbg:?}");
        }
        self.selection = new_sel;
    }
    /// Update the layout.
    fn update_layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<T>) {
        let mut builder =
            layout_cx.ranged_builder(font_cx, &self.buffer, self.scale, self.quantize);
        for prop in self.default_style.inner().values() {
            builder.push_default(prop.to_owned());
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
