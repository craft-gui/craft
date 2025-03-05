use std::hash::Hasher;
use parley::{FontContext, FontStack, Layout, TextStyle};
use peniko::Brush;
use rustc_hash::FxHasher;
use crate::components::component::ComponentOrElement;
use crate::components::ComponentSpecification;
use crate::elements::element::Element;
use crate::elements::layout_context::AvailableSpace;
use crate::elements::Span;
use crate::elements::text::text::TextFragment;
use crate::elements::text::TextState;
use crate::style::Style;

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct TextHashValue {
    pub computed_width: f32,
    pub computed_height: f32,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct TextHashKey {
    pub text_hash: u64,
    pub font_settings_hash: u64,

    // Layout Related Keys
    pub width_constraint: Option<u32>,
    pub height_constraint: Option<u32>,
    pub available_space_width: AvailableSpace,
    pub available_space_height: AvailableSpace,
}

/// Generate a parley TextStyle from our oku::Style struct.
fn style_to_parley_style<'a>(style: &Style) -> TextStyle<'a, Brush> {
    let text_brush = Brush::Solid(style.color());
    let font_stack = FontStack::from("system-ui");

    TextStyle {
        brush: text_brush,
        font_stack,
        line_height: 1.5,
        font_size: style.font_size(),
        ..Default::default()
    }
}

/// Hash our text and font settings from the children and fragments of a Text element.
fn hash_text_and_font_settings_from_text_fragments(root_style: &Style, children: &Vec<ComponentSpecification>, fragments: &Vec<TextFragment>) -> (u64, u64) {
    let mut text_hasher = FxHasher::default();
    let mut font_settings_hasher = FxHasher::default();
    
    let mut hash_font_settings = |style: &Style| {
        font_settings_hasher.write_u8(style.font_family_length());
        font_settings_hasher.write(&style.font_family_raw());
        font_settings_hasher.write_u32(style.font_size().to_bits());
    };
    
    for fragment in fragments.iter() {
        match fragment {
            TextFragment::String(str) => {
                text_hasher.write(str.as_bytes());
                hash_font_settings(root_style);
            }
            TextFragment::Span(span_index) => {
                let span = children.get(*span_index as usize).unwrap();

                match &span.component {
                    ComponentOrElement::Element(ele) => {
                        let ele = &*ele.internal;

                        if let Some(span) = ele.as_any().downcast_ref::<Span>() {
                            text_hasher.write(span.text.as_bytes());
                            hash_font_settings(span.style());
                        }
                    }
                    _ => {}
                }
            }
            TextFragment::InlineComponentSpecification(_inline) => {}
        }
    }

    let text_hash = text_hasher.finish();
    let font_settings_hash = font_settings_hasher.finish();

    (text_hash, font_settings_hash)
}

/// Build a parley text layout tree from the children and fragments of a Text element.
fn build_text_layout_tree<'a>(font_context: &'a mut FontContext, font_layout_context: &'a mut parley::LayoutContext<Brush>,
                          root_style: &'a TextStyle<'a, Brush>,
                          children: &'a Vec<ComponentSpecification>,
                          fragments: &'a Vec<TextFragment>) -> parley::TreeBuilder<'a, Brush> {
    
    let mut builder: parley::TreeBuilder<Brush> = font_layout_context.tree_builder(font_context, 1.0, &root_style);
    for fragment in fragments.iter() {
        match fragment {
            TextFragment::String(str) => {
                builder.push_text(str);
            }
            TextFragment::Span(span_index) => {
                let span = children.get(*span_index as usize).unwrap();

                // Add the span text and their style.
                match &span.component {
                    ComponentOrElement::Element(ele) => {
                        let ele = &*ele.internal;

                        if let Some(span) = ele.as_any().downcast_ref::<Span>() {
                            builder.push_style_span(style_to_parley_style(span.style()));
                            builder.push_text(&span.text);
                            builder.pop_style_span();
                        }
                    }
                    _ => {}
                }
            }
            TextFragment::InlineComponentSpecification(inline) => {}
        }
    }
    
    builder
}

pub(crate) fn recompute_layout_from_cache_key(layout: &mut Layout<Brush>, cache_key: &TextHashKey) {
    let width_constraint = cache_key.width_constraint.map(|w| f32::from_bits(w));
    layout.break_all_lines(width_constraint);
    layout.align(width_constraint, parley::Alignment::Start, parley::AlignmentOptions::default());
}

impl TextState {

    /// Measure the width and height of the text given layout constraints.
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_context: &mut FontContext,
        font_layout_context: &mut parley::LayoutContext<Brush>,
    ) -> taffy::Size<f32> {

        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };
        
        let (text_hash, font_settings_hash) = hash_text_and_font_settings_from_text_fragments(&self.style, &self.children, &self.fragments);
        
        let cache_key = TextHashKey {
            text_hash,
            font_settings_hash,
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
        };

        
        // If the text or font settings have changed since the last cache, we have to recompute the size of our text.
        let mut text_changed = true;
        if let Some(last_cache_key) = &self.last_cache_key {
            if last_cache_key.text_hash == cache_key.text_hash && 
                last_cache_key.font_settings_hash == cache_key.font_settings_hash {
                text_changed = false;
            }
        }
        
        // Update the current cache key.
        self.last_cache_key = Some(cache_key.clone());

        // Use the cached size if possible and if the text/font settings haven't changed.
        if self.cached_text_layout.contains_key(&cache_key) && !text_changed {
            let computed_size = self.cached_text_layout.get(&cache_key).unwrap();

            taffy::Size {
                width: computed_size.computed_width,
                height: computed_size.computed_height,
            }
        } else { // Cache is not available or the text/font settings have changed, so we need to recompute the size.
            let root_style = style_to_parley_style(&self.style);
            
            
            let mut builder = build_text_layout_tree(font_context, font_layout_context, &root_style, &self.children, &self.fragments);
            let (mut layout, _text): (Layout<Brush>, String) = builder.build();
            recompute_layout_from_cache_key(&mut layout, &self.last_cache_key.as_ref().unwrap());

            let width = layout.width().ceil();
            let height = layout.height().ceil();

            let computed_size = TextHashValue {
                computed_width: width,
                computed_height: height,
            };

            // Update the cache.
            self.layout = layout;
            self.cached_text_layout.insert(cache_key.clone(), computed_size);

            taffy::Size {
                width: computed_size.computed_width,
                height: computed_size.computed_height,
            }
        }

    }
}