use crate::elements::layout_context::{MetricsRaw, TextHashKey};
use crate::elements::text::{hash_text, AttributesRaw, TextHashValue};
use crate::style::Style;
use cosmic_text::{Action, Buffer, Edit, Editor, FontSystem, Motion, Shaping};
use std::collections::HashMap;

pub struct CachedEditor<'a> {
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_key: Option<TextHashKey>,
    pub editor: Editor<'a>,
    // Attributes
    pub(crate) attributes: AttributesRaw,
    // Metrics
    pub(crate) metrics: MetricsRaw,
}

impl CachedEditor<'_> {
    pub(crate) fn get_last_cache_entry(&self) -> &TextHashValue {
        let key = self.last_key.unwrap();
        &self.cached_text_layout[&key]
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
    ) -> taffy::Size<f32> {
        let cache_key = TextHashKey::new(known_dimensions, available_space);
        self.last_key = Some(cache_key);

        if self.cached_text_layout.len() > 3 {
            self.cached_text_layout.clear();
        }

        let cached_text_layout_value = self.cached_text_layout.get(&cache_key);

        if let Some(cached_text_layout_value) = cached_text_layout_value {
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics_and_size(font_system, self.metrics.to_metrics(), cache_key.width_constraint.map(f32::from_bits), cache_key.height_constraint.map(f32::from_bits));
            });
            self.editor.shape_as_needed(font_system, true);

            // Determine measured size of text
            let cached_text_layout_value = self.editor.with_buffer(|buffer| {
                let (width, total_lines) = buffer
                    .layout_runs()
                    .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
                let height = total_lines as f32 * buffer.metrics().line_height;

                TextHashValue {
                    computed_width: width,
                    computed_height: height,
                    buffer: buffer.clone(),
                }
            });

            let size = taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            };

            self.cached_text_layout.insert(cache_key, cached_text_layout_value);
            size
        }
    }
    
    pub(crate) fn new( text: &String, style: &Style, scaling_factor: f64, font_system: &mut FontSystem) -> Self {
        let metrics = MetricsRaw::from(style, scaling_factor);

        let buffer = Buffer::new(font_system, metrics.to_metrics());
        let mut editor = Editor::new(buffer);
        editor.borrow_with(font_system);

        let text_hash = hash_text(text);
        let attributes = AttributesRaw::from(style);
        editor.with_buffer_mut(|buffer| buffer.set_text(font_system, text, attributes.to_attrs(), Shaping::Advanced));
        editor.action(font_system, Action::Motion(Motion::End));

        Self {
            text_hash,
            cached_text_layout: HashMap::new(),
            last_key: None,
            editor,
            attributes,
            metrics,
        }
    }
    
    pub(crate) fn update_state(&mut self, text: &String, style: &Style, scaling_factor: f64, reload_fonts: bool, font_system: &mut FontSystem) {
        let text_hash = hash_text(text);
        let attributes = AttributesRaw::from(style);
        let metrics = MetricsRaw::from(style, scaling_factor);

        let text_changed = text_hash != self.text_hash
            || reload_fonts
            || attributes != self.attributes;
        let size_changed = metrics != self.metrics;

        if text_changed || size_changed {
            self.clear_cache();
        }

        if size_changed {
            self.metrics = metrics;
        }

        if text_changed {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_text(font_system, text, attributes.to_attrs(), Shaping::Advanced);
            });

            self.attributes = attributes;
            self.text_hash = text_hash;
        }
    }
    
    pub(crate) fn clear_cache(&mut self) {
        self.cached_text_layout.clear();
        self.last_key = None;
    }
}