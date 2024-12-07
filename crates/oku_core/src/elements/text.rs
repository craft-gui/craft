use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{CommonElementData, Element, ElementBox};
use crate::elements::layout_context::{AvailableSpace, LayoutContext, MetricsDummy, TaffyTextContext, TextHashKey};
use crate::renderer::renderer::Rectangle;
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::style::{Style};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, Size, TaffyTree};
use winit::dpi::{LogicalPosition, PhysicalPosition};
use crate::elements::ElementStyles;

use crate::components::props::Props;

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
}

impl TextState {
    pub(crate) fn new(
        id: ComponentId,
        text_hash: u64,
        buffer: Buffer,
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
            },
        }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
        text_hash: u64,
        metrics: Metrics,
    ) -> Size<f32> {
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
            Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            let cached_text_layout_value = cached_text_layout_value.unwrap();
            Size {
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
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &StateStore,
    ) {
        let bounding_rectangle = Rectangle::new(
            self.common_element_data.computed_x_transformed + self.common_element_data.computed_padding[3],
            self.common_element_data.computed_y_transformed + self.common_element_data.computed_padding[0],
            self.common_element_data.computed_width,
            self.common_element_data.computed_height,
        );
        renderer.draw_rect(bounding_rectangle, self.common_element_data.style.background);
        renderer.draw_text(
            self.common_element_data.component_id,
            bounding_rectangle,
            self.common_element_data.style.color,
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
        scale_factor: f64,
    ) -> NodeId {
        println!("Scaling text by factor: {}", scale_factor);
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        let font_size = PhysicalPosition::from_logical(LogicalPosition::new(self.common_element_data.style.font_size, self.common_element_data.style.font_size), scale_factor).x;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);
        
        let text_state: &mut TextState = _element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap();
        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(TaffyTextContext::new(
                    self.common_element_data.component_id,
                    metrics
                )),
            )
            .unwrap()
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        
        let text_context = self.get_state_mut(element_state);

        let metrics = text_context.last_key;
        
        let metrics = Metrics::new(f32::from_bits(metrics.metrics.font_size), f32::from_bits(metrics.metrics.line_height));
        
        text_context.buffer.set_metrics(font_system, metrics);

        text_context.buffer.set_size(
            font_system,
            text_context.last_key.width_constraint.map(|w| f32::from_bits(w)),
            text_context.last_key.height_constraint.map(|h| f32::from_bits(h)),
        );
        text_context.buffer.shape_until_scroll(font_system, true);

        self.resolve_position(x, y, result);

        self.common_element_data.computed_content_width = result.content_size.width;
        self.common_element_data.computed_content_height = result.content_size.height;
        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;
        self.common_element_data.computed_padding =
            [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];

        let transformed_xy = transform.mul_vec4(glam::vec4(
            self.common_element_data.computed_x,
            self.common_element_data.computed_y,
            0.0,
            1.0,
        ));
        self.common_element_data.computed_x_transformed = transformed_xy.x;
        self.common_element_data.computed_y_transformed = transformed_xy.y;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_system: &mut FontSystem) -> Box<StateStoreItem> {
        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let attributes = Attrs::new();

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(
            font_system,
            &self.text,
            attributes,
            Shaping::Advanced,
        );

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let state = TextState::new(
            self.common_element_data.component_id,
            text_hash,
            buffer,
        );

        Box::new(state)
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut StateStore) {
        let state = self.get_state_mut(element_state);

        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let attributes = Attrs::new();

        if text_hash != state.text_hash {
            state.text_hash = state.text_hash;
            state.buffer.set_text(
                font_system,
                &self.text,
                attributes,
                Shaping::Advanced,
            );
        }
    }
    
}

impl Text {
    
    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}