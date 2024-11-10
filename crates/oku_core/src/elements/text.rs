use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle};
use crate::components::component::{ComponentId, ComponentSpecification, GenericUserState};
use crate::elements::element::{CommonElementData, Element, ElementState};
use crate::elements::layout_context::{
    AvailableSpace, LayoutContext, MetricsDummy, TaffyTextContext, TextHashKey,
};
use crate::style::{
    AlignItems, Display, FlexDirection, FontStyle, JustifyContent, Unit, Weight,
};
use crate::RendererBox;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, Size, TaffyTree};
use crate::components::UpdateResult;
use crate::engine::events::OkuEvent;

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
    pub id: ComponentId,
    pub buffer: Buffer,
    pub metrics: Metrics,
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub color: cosmic_text::Color,
    pub last_key: TextHashKey,
}

impl TextState {
    pub(crate) fn new(
        id: ComponentId,
        metrics: Metrics,
        text_hash: u64,
        buffer: Buffer,
        color: Option<cosmic_text::Color>,
    ) -> Self {
        Self {
            id,
            metrics,
            buffer,
            text_hash,
            cached_text_layout: Default::default(),
            color: color.unwrap_or(cosmic_text::Color::rgb(0, 0, 0)),
            last_key: TextHashKey {
                text_hash,
                width_constraint: None,
                height_constraint: None,
                available_space_width: AvailableSpace::MinContent,
                available_space_height: AvailableSpace::MinContent,
                metrics: MetricsDummy {
                    font_size: metrics.font_size.to_bits(),
                    line_height: metrics.line_height.to_bits(),
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
            self.buffer.set_metrics(font_system, self.metrics);
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
    fn get_state<'a>(&self, element_state: &'a mut HashMap<ComponentId, Box<ElementState>>) -> &'a TextState {
        element_state.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(
        &self,
        element_state: &'a mut HashMap<ComponentId, Box<ElementState>>,
    ) -> &'a mut TextState {
        element_state.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap()
    }
}

impl Element for Text {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
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
        transform: glam::Mat4,
        element_state: &HashMap<ComponentId, Box<ElementState>>,
    ) {
        let bounding_rectangle = Rectangle::new(
            self.common_element_data.computed_x + self.common_element_data.computed_padding[3],
            self.common_element_data.computed_y + self.common_element_data.computed_padding[0],
            self.common_element_data.computed_width,
            self.common_element_data.computed_height,
        );
        renderer.draw_rect(bounding_rectangle, self.common_element_data.style.background, transform);

        renderer.draw_text(
            self.common_element_data.component_id,
            bounding_rectangle,
            self.common_element_data.style.color,
            transform
        );
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem, element_state: &mut HashMap<ComponentId, Box<GenericUserState>>) -> NodeId {
        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);
        let mut attributes = Attrs::new();
        attributes = attributes.style(match self.common_element_data.style.font_style {
            FontStyle::Normal => cosmic_text::Style::Normal,
            FontStyle::Italic => cosmic_text::Style::Italic,
            FontStyle::Oblique => cosmic_text::Style::Oblique,
        });

        attributes.weight = cosmic_text::Weight(self.common_element_data.style.font_weight.0);
        let style: taffy::Style = self.common_element_data.style.into();

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(TaffyTextContext::new(
                    self.common_element_data.component_id,
                    metrics,
                    self.text.clone(),
                    text_hash,
                    attributes,
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
        font_system: &mut FontSystem,
        element_state: &mut HashMap<ComponentId, Box<ElementState>>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();

        let text_context = self.get_state_mut(element_state);

        text_context.buffer.set_metrics(font_system, text_context.metrics);

        text_context.buffer.set_size(
            font_system,
            text_context.last_key.width_constraint.map(|w| f32::from_bits(w)),
            text_context.last_key.height_constraint.map(|h| f32::from_bits(h)),
        );
        text_context.buffer.shape_until_scroll(font_system, true);

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;

        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;

        self.common_element_data.computed_padding =
            [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}

impl Text {
    pub fn add_child(self, _widget: Box<dyn Element>) -> Self {
        panic!("Text can't have children.");
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Text {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Self {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.common_element_data.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.common_element_data.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Self {
        self.common_element_data.style.font_weight = font_weight;
        self
    }

    pub const fn font_style(mut self, font_style: FontStyle) -> Self {
        self.common_element_data.style.font_style = font_style;
        self
    }

    pub const fn display(mut self, display: Display) -> Self {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Self {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Self {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Self {
        self.common_element_data.style.height = height;
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }
    
    pub fn component(self) -> ComponentSpecification {
        ComponentSpecification::new(self.into())
    }
}
