use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::{CosmicTextContent, LayoutContext};
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, Weight};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics};
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    text_buffer: Option<Buffer>,
    common_element_data: CommonElementData
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_string(),
            text_buffer: None,
            common_element_data: Default::default(),
        }
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

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        let text_color = cosmic_text::Color::rgba(
            self.common_element_data.style.color.r_u8(),
            self.common_element_data.style.color.g_u8(),
            self.common_element_data.style.color.b_u8(),
            self.common_element_data.style.color.a_u8(),
        );

        if self.text_buffer.is_none() {
            return;
        }
        
        let element_x = self.common_element_data().computed_x;
        let element_y = self.common_element_data().computed_y;
        let text_buffer = self.text_buffer.as_mut().unwrap();

        renderer.draw_rect(
            Rectangle::new(self.common_element_data.computed_x, self.common_element_data.computed_y, self.common_element_data.computed_width, self.common_element_data.computed_height),
            self.common_element_data.style.background,
        );

        text_buffer.draw(
            &mut render_context.font_system,
            &mut render_context.swash_cache,
            text_color,
            |x, y, w, h, color| {
                let r = color.r();
                let g = color.g();
                let b = color.b();
                let a = color.a();
                let a1 = a as f32 / 255.0;
                let a2 = self.common_element_data.style.color.a / 255.0;
                let a = (a1 * a2 * 255.0) as u8;

                let p_x: i32 = (element_x + self.common_element_data.computed_padding[3] + x as f32) as i32;
                let p_y: i32 = (element_y + self.common_element_data.computed_padding[0] + y as f32) as i32;

                renderer.draw_rect(
                    Rectangle::new(p_x as f32, p_y as f32, w as f32, h as f32),
                    Color::new_from_rgba_u8(r, g, b, a),
                );
            },
        );
    }

    fn debug_draw(&mut self, _render_context: &mut RenderContext) {}

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);
        let mut attrs = Attrs::new();

        attrs.weight = cosmic_text::Weight(self.common_element_data.style.font_weight.0);
        let style: taffy::Style = self.common_element_data.style.into();

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(CosmicTextContent::new(metrics, self.text.as_str(), attrs, font_system)),
            )
            .unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();
        let buffer = taffy_tree.get_node_context(root_node).unwrap();

        if let LayoutContext::Text(cosmic_text) = buffer { self.text_buffer = Option::from(cosmic_text.buffer.clone()) }

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;

        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;

        self.common_element_data.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
    }
}

impl Text {
    pub fn add_child(self, _widget: Box<dyn Element>) -> Text {
        panic!("Text can't have children.");
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Text {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Text {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Text {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Text {
        self.common_element_data.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Text {
        self.common_element_data.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Text {
        self.common_element_data.style.font_weight = font_weight;
        self
    }

    pub const fn display(mut self, display: Display) -> Text {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Text {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Text {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Text {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Text {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Text {
        self.common_element_data.style.height = height;
        self
    }
}
