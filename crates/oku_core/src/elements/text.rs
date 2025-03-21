use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, MetricsRaw, TaffyTextContext, TextHashKey};
use crate::elements::ElementStyles;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Shaping, Weight};
use rustc_hash::FxHasher;
use std::any::Any;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};

use crate::components::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::geometry::Point;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    common_element_data: CommonElementData,
}

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
        let font_family = if style.font_family_length() == 0 {
            None
        } else {
            Some(style.font_family_raw())
        };
        Self {
            font_family_length: style.font_family_length(),
            font_family,
            weight: Weight(style.font_weight().0),
        }
    }

    pub(crate) fn to_attrs(&self) -> Attrs {
        let mut attrs = Attrs::new();
        if let Some(font_family) = &self.font_family {
            attrs.family = Family::Name(
                std::str::from_utf8(&font_family[..self.font_family_length as usize]).unwrap()
            );
            attrs.weight = self.weight;
        }
        attrs
    }

}

pub struct TextState {
    #[allow(dead_code)]
    pub id: ComponentId,
    buffer: Buffer,
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_key: Option<TextHashKey>,
    // Attributes
    pub attributes: AttributesRaw,
    // Metrics
    pub metrics: MetricsRaw,
}

impl TextState {
    pub(crate) fn get_last_cache_entry(&self) -> &TextHashValue {
        let key = self.last_key.unwrap();
        &self.cached_text_layout[&key]
    }
}

pub(crate) fn hash_text(text: &String) -> u64 {
    let mut text_hasher = FxHasher::default();
    text_hasher.write(text.as_ref());
    text_hasher.finish()
}

impl TextState {
    pub(crate) fn new(
        id: ComponentId,
        text_hash: u64,
        buffer: Buffer,
        metrics: MetricsRaw,
        attributes: AttributesRaw,
    ) -> Self {
        Self {
            id,
            buffer,
            text_hash,
            cached_text_layout: Default::default(),
            last_key: None,
            attributes,
            metrics,
        }
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
            self.buffer.set_metrics_and_size(font_system, self.metrics.to_metrics(), cache_key.width_constraint.map(f32::from_bits), cache_key.height_constraint.map(f32::from_bits));
            self.buffer.shape_until_scroll(font_system, true);

            // Determine measured size of text
            let (width, total_lines) = self
                .buffer
                .layout_runs()
                .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
            let height = total_lines as f32 * self.buffer.metrics().line_height;

            let cached_text_layout_value = TextHashValue {
                computed_width: width,
                computed_height: height,
                buffer: self.buffer.clone(),
            };

            let size = taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            };

            self.cached_text_layout.insert(cache_key, cached_text_layout_value);

            size
        }
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_string(),
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut TextState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }
}

impl PartialEq for AttributesRaw {
    fn eq(&self, other: &Self) -> bool {
        self.font_family == other.font_family &&
            self.font_family_length == other.font_family_length &&
            self.weight == other.weight
    }
}

impl Element for Text {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(TaffyTextContext::new(self.common_element_data.component_id)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
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
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let metrics = MetricsRaw::from(&self.common_element_data.style, scaling_factor);
        let attributes = AttributesRaw::from(&self.common_element_data.style);

        let mut buffer = Buffer::new(font_system, metrics.to_metrics());
        buffer.set_text(font_system, &self.text, attributes.to_attrs(), Shaping::Advanced);
        let text_hash = hash_text(&self.text);

        let state = TextState::new(
            self.common_element_data.component_id,
            text_hash,
            buffer,
            metrics,
            attributes,
        );

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(state)
        }
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let state = self.get_state_mut(element_state);

        let text_hash = hash_text(&self.text);
        let attributes = AttributesRaw::from(&self.common_element_data.style);
        let metrics = MetricsRaw::from(&self.common_element_data.style, scaling_factor);

        let text_changed = text_hash != state.text_hash
            || reload_fonts
            || attributes != state.attributes;
        let size_changed = metrics != state.metrics;

        if text_changed || size_changed {
            state.cached_text_layout.clear();
            state.last_key = None;
        }

        if size_changed {
            state.metrics = metrics;
        }

        if text_changed {
            state.buffer.set_text(font_system, &self.text, attributes.to_attrs(), Shaping::Advanced);
            state.attributes = attributes;
            state.text_hash = text_hash;
        }
    }
}

impl Text {

    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
