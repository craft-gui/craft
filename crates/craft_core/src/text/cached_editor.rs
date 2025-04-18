use crate::elements::layout_context::{MetricsRaw, TextHashKey};
use crate::style::Style;
use cosmic_text::{Attrs, Buffer, Edit, Editor, Family, FontSystem, Weight};
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use winit::event::Modifiers;

#[derive(Clone)]
pub struct TextHashValue {
    pub computed_width: f32,
    pub computed_height: f32,
    pub buffer: Buffer,
}

pub struct AttributesRaw {
    pub(crate) font_family_length: u8,
    pub(crate) font_family: Option<[u8; 64]>,
    weight: Weight,
}

impl AttributesRaw {
    pub(crate) fn from(style: &Style) -> Self {
        let font_family = if style.font_family_length() == 0 { None } else { Some(style.font_family_raw()) };
        Self {
            font_family_length: style.font_family_length(),
            font_family,
            weight: Weight(style.font_weight().0),
        }
    }

    pub(crate) fn to_attrs(&self) -> Attrs {
        let mut attrs = Attrs::new();
        if let Some(font_family) = &self.font_family {
            attrs.family = Family::Name(std::str::from_utf8(&font_family[..self.font_family_length as usize]).unwrap());
            attrs.weight = self.weight;
        }
        attrs
    }
}

impl PartialEq for AttributesRaw {
    fn eq(&self, other: &Self) -> bool {
        self.font_family == other.font_family
            && self.font_family_length == other.font_family_length
            && self.weight == other.weight
    }
}

pub(crate) fn hash_text(text: &String) -> u64 {
    let mut text_hasher = FxHasher::default();
    text_hasher.write(text.as_ref());
    text_hasher.finish()
}

pub struct CachedEditor<'a> {
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    /// The key to get the last computed `Buffer` and its width and height.
    pub last_key: Option<TextHashKey>,
    /// The internal cosmic-text editor that we wrap, so that we can do caching.
    pub editor: Editor<'a>,
    pub modifiers: Modifiers,
    /// Stores Attrs fields as integers for hashing.
    pub(crate) attributes: AttributesRaw,
    /// Stores Metric fields as integers for hashing.
    pub(crate) metrics: MetricsRaw,
    pub(crate) dragging: bool,
    /// Whether the cache needs to rebuilt or not.
    pub(crate) is_dirty: bool,
}

impl CachedEditor<'_> {
    /// Get the last cache entry using the `last_key`.
    pub(crate) fn get_last_cache_entry(&self) -> &TextHashValue {
        let key = self.last_key.unwrap();
        &self.cached_text_layout[&key]
    }

    /// Measure the width and height of the text and cache the result.
    /// This method may be called up to 3-5 times by Taffy for a single Text element.
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
    ) -> taffy::Size<f32> {
        let cache_key = TextHashKey::new(known_dimensions, available_space);
        self.rebuild_cache_if_needed(cache_key, font_system)
    }

    pub(crate) fn new(text: &String, style: &Style, scaling_factor: f64, font_system: &mut FontSystem) -> Self {
        let attributes = AttributesRaw::from(style);
        let metrics = MetricsRaw::from(style, scaling_factor);

        let buffer = Buffer::new(font_system, metrics.to_metrics());
        let mut editor = Editor::new(buffer);
        editor.borrow_with(font_system);

        let mut cached_editor = Self {
            text_hash: 0,
            cached_text_layout: HashMap::new(),
            last_key: None,
            editor,
            modifiers: Default::default(),
            attributes,
            metrics,
            dragging: false,
            is_dirty: true,
        };

        cached_editor.action_set_text(font_system, Some(text));
        cached_editor.move_to_end(font_system);

        cached_editor
    }

    pub(crate) fn update_state(
        &mut self,
        text: Option<&String>,
        style: &Style,
        scaling_factor: f64,
        reload_fonts: bool,
        font_system: &mut FontSystem,
    ) {
        let attributes = AttributesRaw::from(style);
        let metrics = MetricsRaw::from(style, scaling_factor);

        self.action_set_attributes(attributes);
        self.action_set_reload_fonts(reload_fonts);
        self.action_set_text(font_system, text);
        self.action_set_metrics(metrics);
    }

    /// Get the current text, INCLUDING the IME pre-edit text.
    pub(crate) fn get_text(&mut self) -> String {
        self.editor.with_buffer(|buffer| {
            let mut buffer_string: String = String::new();
            for line in buffer.lines.iter() {
                buffer_string.push_str(line.text());
                buffer_string.push_str(line.ending().as_str());
            }
            buffer_string
        })
    }

    pub(crate) fn is_control_or_super_modifier_pressed(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.modifiers.state().super_key()
        } else {
            self.modifiers.state().control_key()
        }
    }

    pub(crate) fn rebuild_cache_if_needed(
        &mut self,
        cache_key: TextHashKey,
        font_system: &mut FontSystem,
    ) -> taffy::Size<f32> {
        self.last_key = Some(cache_key);
        // Currently we are caching the `Buffer` which is memory hungry, so we should clear the cache if this grows to be too big.
        // Measure is called 3-5 times.
        if self.cached_text_layout.len() > 3 {
            self.cached_text_layout.clear();
        }
        if self.is_dirty {
            self.cached_text_layout.clear();
            self.is_dirty = false;
        }
        let cached_text_layout_value = self.cached_text_layout.get(&cache_key);

        if let Some(cached_text_layout_value) = cached_text_layout_value {
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics_and_size(
                    font_system,
                    self.metrics.to_metrics(),
                    cache_key.width_constraint.map(f32::from_bits),
                    cache_key.height_constraint.map(f32::from_bits),
                );
            });
            self.editor.shape_as_needed(font_system, true);

            // Measure the size of the text.
            let cached_text_layout_value = self.editor.with_buffer(|buffer| {
                let (width, total_lines) = buffer
                    .layout_runs()
                    .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
                let height = total_lines as f32 * buffer.metrics().line_height;

                let mut buffer = buffer.clone();
                buffer.set_redraw(false);

                TextHashValue {
                    computed_width: width,
                    computed_height: height,
                    buffer,
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
}
