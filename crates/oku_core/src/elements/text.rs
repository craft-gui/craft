use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{AvailableSpace, LayoutContext, MetricsDummy, TaffyTextContext, TextHashKey};
use crate::elements::ElementStyles;
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{LogicalPosition, PhysicalPosition};

use crate::components::props::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::geometry::{Border, ElementRectangle, Margin, Padding, Point, Size};

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    common_element_data: CommonElementData,
}

#[derive(Copy, Clone)]
pub struct TextHashValue {
    pub computed_width: f32,
    pub computed_height: f32,
}

pub struct TextState {
    #[allow(dead_code)]
    pub id: ComponentId,
    pub buffer: Buffer,
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_key: TextHashKey,

    pub(crate) font_family_length: u8,
    pub(crate) font_family: [u8; 64],
    weight: Weight,
}

impl TextState {
    pub(crate) fn new(
        id: ComponentId,
        text_hash: u64,
        buffer: Buffer,
        font_family_length: u8,
        font_family: [u8; 64],
        weight: Weight,
    ) -> Self {
        Self {
            id,
            buffer,
            text_hash,
            cached_text_layout: Default::default(),
            last_key: TextHashKey {
                text_hash,
                width_constraint: None,
                height_constraint: None,
                available_space_width: AvailableSpace::MinContent,
                available_space_height: AvailableSpace::MinContent,
                metrics: MetricsDummy {
                    font_size: 0,
                    line_height: 0,
                },
                font_family_length,
                font_family,
            },
            font_family_length,
            font_family,
            weight,
        }
    }

    pub fn font_family(&self) -> Option<&str> {
        if self.font_family_length == 0 {
            None
        } else {
            Some(std::str::from_utf8(&self.font_family[..self.font_family_length as usize]).unwrap())
        }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
        text_hash: u64,
        metrics: Metrics,
        font_family_length: u8,
        font_family: [u8; 64],
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

        let key = TextHashKey {
            text_hash,
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
            metrics: MetricsDummy {
                font_size: metrics.font_size.to_bits(),
                line_height: metrics.line_height.to_bits(),
            },
            font_family_length,
            font_family,
        };

        self.last_key = key;
        let cached_text_layout_value = self.cached_text_layout.get(&key);
        self.text_hash = text_hash;

        if cached_text_layout_value.is_none() {
            self.buffer.set_metrics(font_system, metrics);
            self.buffer.set_size(font_system, width_constraint, height_constraint);
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
            };

            self.cached_text_layout.insert(key, cached_text_layout_value);
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            let cached_text_layout_value = cached_text_layout_value.unwrap();
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
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
    fn get_state<'a>(&self, element_state: &'a StateStore) -> &'a TextState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut StateStore) -> &'a mut TextState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap()
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
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &StateStore,
        pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed.clone();
        let border_rectangle = computed_layer_rectangle_transformed.border_rectangle();
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color,
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        let font_size = PhysicalPosition::from_logical(
            LogicalPosition::new(self.common_element_data.style.font_size, self.common_element_data.style.font_size),
            scale_factor,
        )
        .x;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);



        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(TaffyTextContext::new(self.common_element_data.component_id, metrics)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        z_index: &mut u32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
        _pointer: Option<Point>,
    ) {
        let text_context = self.get_state_mut(element_state);

        let metrics = text_context.last_key;
        let metrics =
            Metrics::new(f32::from_bits(metrics.metrics.font_size), f32::from_bits(metrics.metrics.line_height));

        text_context.buffer.set_metrics(font_system, metrics);

        text_context.buffer.set_size(
            font_system,
            text_context.last_key.width_constraint.map(|w| f32::from_bits(w)),
            text_context.last_key.height_constraint.map(|h| f32::from_bits(h)),
        );
        text_context.buffer.shape_until_scroll(font_system, true);

        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(x, y, transform, result, z_index);
        
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_system: &mut FontSystem) -> Box<StateStoreItem> {
        let metrics = Metrics::new(12.0, 12.0);

        let mut attributes = Attrs::new();

        let new_font_family = self.common_element_data.style.font_family();

        if let Some(family) = new_font_family {
            attributes.family = Family::Name(family);
        }

        attributes.weight = Weight(self.common_element_data.style.font_weight.0);

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let state = TextState::new(
            self.common_element_data.component_id,
            text_hash,
            buffer,
            self.common_element_data.style.font_family_length,
            self.common_element_data.style.font_family,
            attributes.weight,
        );

        Box::new(state)
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut StateStore, reload_fonts: bool) {
        let state = self.get_state_mut(element_state);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let mut attributes = Attrs::new();

        attributes.weight = Weight(self.common_element_data.style.font_weight.0);

        let new_font_family = self.common_element_data.style.font_family();

        if let Some(family) = new_font_family {
            attributes.family = Family::Name(family);
        }

        if text_hash != state.text_hash
            || state.font_family() != new_font_family
            || reload_fonts
            || attributes.weight != state.weight
        {
            state.font_family_length = self.common_element_data.style.font_family_length;
            state.font_family = self.common_element_data.style.font_family;
            state.text_hash = text_hash;
            state.weight = attributes.weight;
            
            state.buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced);
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
