use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::platform::resource_manager::ResourceIdentifier;
use crate::components::component::{ComponentId, ComponentSpecification, GenericUserState};
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit, Weight};
use crate::RendererBox;
use cosmic_text::FontSystem;
use log::info;
use std::any::Any;
use std::collections::HashMap;
use taffy::{NodeId, TaffyTree};
use crate::engine::events::OkuEvent;

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub common_element_data: CommonElementData,
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Image {
        Image {
            resource_identifier,
            common_element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Image"
    }
}

impl Element for Image {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Image"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
    ) {
        info!("trying to draw image: {:?}", self.common_element_data.computed_height);
        renderer.draw_image(
            Rectangle::new(
                self.common_element_data.computed_x,
                self.common_element_data.computed_y,
                self.common_element_data.computed_width,
                self.common_element_data.computed_height,
            ),
            self.resource_identifier.clone(),
        );
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, _font_system: &mut FontSystem) -> NodeId {
        let style: taffy::Style = self.common_element_data.style.into();
        
        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Image(ImageContext {
                    resource_identifier: self.resource_identifier.clone(),
                }),
            )
            .unwrap()
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        _font_system: &mut FontSystem,
        _element_state: &mut HashMap<ComponentId, Box<GenericUserState>>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();

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

    fn on_event(&self, event: OkuEvent, element_state: &mut HashMap<ComponentId, Box<GenericUserState>>) {
    }
}

impl Image {
    pub fn add_child(self, _widget: Box<dyn Element>) -> Image {
        panic!("Text can't have children.");
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Image {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Image {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Image {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Image {
        self.common_element_data.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Image {
        self.common_element_data.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Image {
        self.common_element_data.style.font_weight = font_weight;
        self
    }

    pub const fn display(mut self, display: Display) -> Image {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Image {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Image {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Image {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Image {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Image {
        self.common_element_data.style.height = height;
        self
    }

    pub const fn max_width(mut self, max_width: Unit) -> Image {
        self.common_element_data.style.max_width = max_width;
        self
    }

    pub const fn max_height(mut self, max_height: Unit) -> Image {
        self.common_element_data.style.max_height = max_height;
        self
    }

    pub const fn computed_x(&self) -> f32 {
        self.common_element_data.computed_x
    }

    pub const fn computed_y(&self) -> f32 {
        self.common_element_data.computed_y
    }

    pub const fn computed_width(&self) -> f32 {
        self.common_element_data.computed_width
    }

    pub const fn computed_height(&self) -> f32 {
        self.common_element_data.computed_height
    }
    pub const fn computed_padding(&self) -> [f32; 4] {
        self.common_element_data.computed_padding
    }

    pub fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= self.common_element_data.computed_x
            && x <= self.common_element_data.computed_x + self.common_element_data.computed_width
            && y >= self.common_element_data.computed_y
            && y <= self.common_element_data.computed_y + self.common_element_data.computed_height
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    pub fn component(self) -> ComponentSpecification {
        ComponentSpecification::new(self.into())
    }
}
