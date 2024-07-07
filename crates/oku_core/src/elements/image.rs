use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, Weight};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Image {
    image_path: String,
    common_element_data: CommonElementData,
}

impl Image {
    pub fn new(image_path: &str) -> Image {
        Image {
            image_path: image_path.to_string(),
            common_element_data: Default::default(),
        }
    }
}

impl Element for Image {
    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children(&self) -> Vec<Box<dyn Element>> {
        Vec::new()
    }

    fn children_as_ref<'a>(&'a self) -> Vec<&'a dyn Element> {
        Vec::new()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "Image"
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, _render_context: &mut RenderContext) {
        renderer.draw_image(
            Rectangle::new(self.common_element_data.computed_x, self.common_element_data.computed_y, self.common_element_data.computed_width, self.common_element_data.computed_height),
            self.image_path.as_str(),
        );
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, _font_system: &mut FontSystem) -> NodeId {
        let style: taffy::Style = self.common_element_data.style.into();

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Image(ImageContext {
                    width: 200.0,
                    height: 200.0,
                }),
            )
            .unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;

        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;

        self.common_element_data.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
    }

    fn computed_style(&self) -> Style {
        self.common_element_data.style
    }
    fn computed_style_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= self.common_element_data.computed_x
            && x <= self.common_element_data.computed_x + self.common_element_data.computed_width
            && y >= self.common_element_data.computed_y
            && y <= self.common_element_data.computed_y + self.common_element_data.computed_height
    }

    fn id(&self) -> &Option<String> {
        &self.common_element_data.id
    }

    fn set_id(&mut self, id: Option<String>) {
        self.common_element_data.id = id;
    }

    fn component_id(&self) -> u64 {
        self.common_element_data.component_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.common_element_data.component_id = id;
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

    pub fn computed_style(&self) -> Style {
        self.common_element_data.style
    }
    pub fn computed_style_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }

    pub fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= self.common_element_data.computed_x
            && x <= self.common_element_data.computed_x + self.common_element_data.computed_width
            && y >= self.common_element_data.computed_y
            && y <= self.common_element_data.computed_y + self.common_element_data.computed_height
    }
}
