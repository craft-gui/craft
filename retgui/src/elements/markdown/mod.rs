use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;

use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

use retgui_primitives::brush::Brush;

use retgui_resource_manager::ResourceId;
use retgui_resource_manager::resource_type::ResourceType;

use crate::elements::codeeditor::CodeEditorElement;
use crate::elements::internal_helpers::push_child_to_element;
use crate::elements::{ContainerElement, DynElement, ElementIds, ElementInternals, ImageElement, RetGuiAccessTree, RetainedElements, TextElement, TextInputElement};
use crate::layout::GummyTree;
use crate::style::{Display, FlexDirection, FontStyle, FontWeight, TextStyleProperty, Unit};
use crate::text::RangedStyles;
use crate::{App, Color, pct, px, rgb};

struct StyledText {
    pub text: String,
    pub style: RangedStyles,
}

impl StyledText {
    pub fn new() -> Self {
        StyledText {
            text: String::new(),
            style: RangedStyles::default(),
        }
    }
}

struct MarkdownRenderer<'a, 'elements> {
    elements: &'elements mut RetainedElements,
    gummy_tree: &'elements mut GummyTree,
    access_tree: &'elements RetGuiAccessTree,
    by_internal_id: &'elements mut ElementIds,
    pending_resources: &'elements mut VecDeque<(ResourceId, ResourceType)>,
    element_stack: Vec<DynElement>,
    list_ids: Vec<Option<u64>>,
    styled_text: StyledText,
    bold: Option<usize>,
    font_size: Option<usize>,
    italic: Option<usize>,
    link: Option<(usize, String)>,
    code_block_kind: Option<pulldown_cmark::CodeBlockKind<'a>>,
}

impl<'a, 'elements> MarkdownRenderer<'a, 'elements> {
    pub fn new(
        elements: &'elements mut RetainedElements,
        gummy_tree: &'elements mut GummyTree,
        access_tree: &'elements RetGuiAccessTree,
        by_internal_id: &'elements mut ElementIds,
        pending_resources: &'elements mut VecDeque<(ResourceId, ResourceType)>,
    ) -> Self {
        let root = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
        elements.get_mut(root).set_display(gummy_tree, Display::Block);
        MarkdownRenderer {
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            pending_resources,
            element_stack: vec![root],
            list_ids: Vec::new(),
            styled_text: StyledText {
                text: String::new(),
                style: RangedStyles::default(),
            },
            bold: None,
            font_size: None,
            italic: None,
            link: None,
            code_block_kind: None,
        }
    }

    pub fn push(&mut self, component_specification: DynElement) {
        let parent = self.current_element();
        push_child_to_element(self.elements, self.gummy_tree, parent, component_specification);
    }

    pub fn push_list_id(&mut self, id: Option<u64>) {
        self.list_ids.push(id);
    }

    pub fn pop_list_id(&mut self) -> Option<u64> {
        self.list_ids.pop().expect("List IDs stack should not be empty")
    }

    pub fn list_id(&self) -> Option<u64> {
        *self.list_ids.last().expect("List IDs stack should not be empty")
    }

    pub fn push_container(&mut self, container: DynElement) {
        self.element_stack.push(container);
    }

    pub fn pop_container(&mut self) {
        let container = self.element_stack.pop().expect("Element stack should not be empty");
        self.push(container);
    }

    pub fn current_element(&mut self) -> DynElement {
        *self.element_stack.last().expect("Element stack should not be empty")
    }

    pub fn push_text(&mut self, text: &str) {
        self.styled_text.text.push_str(text);
    }

    pub fn push_rich_text(&mut self, text_input: Option<DynElement>) {
        if self.styled_text.text.is_empty() {
            return;
        }

        let text = if let Some(text_input) = text_input {
            self.elements
                .get_as_mut::<TextInputElement>(text_input)
                .set_text(self.gummy_tree, &self.styled_text.text);
            text_input
        } else {
            let text_input = TextInputElement::create(
                self.elements,
                self.gummy_tree,
                self.access_tree,
                self.by_internal_id,
                &self.styled_text.text,
            );
            let text = self.elements.get_as_mut::<TextInputElement>(text_input);
            text.set_display(self.gummy_tree, Display::Block);
            text.set_border_width_all(self.gummy_tree, px(0));
            text.disabled(true);
            text_input
        };

        self.elements
            .get_as_mut::<TextInputElement>(text)
            .set_ranged_styles(self.gummy_tree, self.styled_text.style.clone());
        self.push(text);
        self.styled_text = StyledText::new();
    }

    pub fn push_link(&mut self, url: String) {
        self.link = Some((self.styled_text.text.len(), url));
    }

    pub fn pop_link(&mut self) {
        if let Some((link_start, link)) = &self.link {
            let end = self.styled_text.text.len();
            self.styled_text
                .style
                .styles
                .push((*link_start..end, TextStyleProperty::Link(link.clone())));
            self.styled_text
                .style
                .styles
                .push((*link_start..end, TextStyleProperty::Color(Brush::Color(rgb(0, 0, 238)))));
            self.link = None;
        }
    }

    pub fn push_bold(&mut self) {
        self.bold = Some(self.styled_text.text.len())
    }

    pub fn push_italic(&mut self) {
        self.italic = Some(self.styled_text.text.len());
    }

    pub fn pop_bold(&mut self) {
        if let Some(bold_start) = self.bold {
            let end = self.styled_text.text.len();
            self.styled_text
                .style
                .styles
                .push((bold_start..end, TextStyleProperty::FontWeight(FontWeight::BOLD)));
            self.bold = None;
        }
    }

    pub fn pop_italic(&mut self) {
        if let Some(start) = self.italic {
            let end = self.styled_text.text.len();
            self.styled_text
                .style
                .styles
                .push((start..end, TextStyleProperty::FontStyle(FontStyle::Italic)));
            self.italic = None;
        }
    }
}

pub fn render_markdown(app: &mut App, markdown: &str) -> DynElement {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut renderer = MarkdownRenderer::new(
        &mut app.elements,
        &mut app.gummy_tree,
        &app.access_tree,
        &mut app.by_internal_id,
        &mut app.pending_resources,
    );

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { .. } => {
                    renderer.push_bold();
                    renderer.font_size = Some(renderer.styled_text.text.len());
                }
                Tag::BlockQuote(_) => {}
                Tag::CodeBlock(code_block_kind) => {
                    renderer.code_block_kind = Some(code_block_kind);
                }
                Tag::HtmlBlock => {}
                Tag::List(item) => {
                    renderer.push_rich_text(None);
                    let children_count = renderer.list_ids.len();
                    renderer.push_list_id(item);
                    let padding = if children_count == 0 { 0 } else { 20 };
                    let list = ContainerElement::create(
                        renderer.elements,
                        renderer.gummy_tree,
                        renderer.access_tree,
                        renderer.by_internal_id,
                    );
                    renderer
                        .elements
                        .get_mut(list)
                        .set_display(renderer.gummy_tree, Display::Flex);
                    renderer
                        .elements
                        .get_mut(list)
                        .set_flex_direction(renderer.gummy_tree, FlexDirection::Column);
                    renderer
                        .elements
                        .get_mut(list)
                        .set_margin(renderer.gummy_tree, px(0), px(0), px(0), px(padding));
                    renderer.push_container(list)
                }
                Tag::Item => {
                    if let Some(id) = renderer.list_id() {
                        let current = renderer.current_element();
                        let offset = renderer.elements.get(current).element_data().children.len() as u64;
                        renderer.push_text(&format!("{}. ", id + offset));
                    } else {
                        renderer.push_text("• ");
                    }
                    let item_container = ContainerElement::create(
                        renderer.elements,
                        renderer.gummy_tree,
                        renderer.access_tree,
                        renderer.by_internal_id,
                    );
                    renderer
                        .elements
                        .get_mut(item_container)
                        .set_display(renderer.gummy_tree, Display::Block);
                    renderer
                        .elements
                        .get_mut(item_container)
                        .set_border_width_all(renderer.gummy_tree, px(0));
                    renderer.push_container(item_container);
                }
                Tag::Emphasis => {
                    renderer.push_italic();
                }
                Tag::Strong => {
                    renderer.push_bold();
                }
                Tag::Strikethrough => {}
                Tag::Superscript => {}
                Tag::Subscript => {}
                Tag::Link { dest_url, .. } => {
                    renderer.push_link(dest_url.to_string());
                }
                Tag::Image { dest_url, .. } => {
                    let resource = if dest_url.starts_with("http") {
                        ResourceId::Url(dest_url.to_string())
                    } else {
                        ResourceId::File(PathBuf::from_str(&dest_url).expect("Invalid file path for image"))
                    };
                    let image = ImageElement::insert(
                        renderer.elements,
                        renderer.gummy_tree,
                        renderer.access_tree,
                        renderer.by_internal_id,
                        renderer.pending_resources,
                        resource,
                    );
                    renderer
                        .elements
                        .get_mut(image)
                        .set_width(renderer.gummy_tree, Unit::Auto);
                    renderer
                        .elements
                        .get_mut(image)
                        .set_height(renderer.gummy_tree, Unit::Auto);
                    let image_container = ContainerElement::create(
                        renderer.elements,
                        renderer.gummy_tree,
                        renderer.access_tree,
                        renderer.by_internal_id,
                    );
                    push_child_to_element(renderer.elements, renderer.gummy_tree, image_container, image);
                    renderer.push(image_container)
                }
                _ => {}
            },
            Event::End(tag) => {
                match tag {
                    TagEnd::Paragraph => {
                        renderer.push_rich_text(None);
                    }
                    TagEnd::Heading(level) => {
                        if let Some(font_size) = renderer.font_size {
                            let size = match level {
                                HeadingLevel::H1 => 32.0,
                                HeadingLevel::H2 => 24.0,
                                HeadingLevel::H3 => 20.0,
                                HeadingLevel::H4 => 18.0,
                                HeadingLevel::H5 => 16.0,
                                HeadingLevel::H6 => 14.0,
                            };
                            renderer.styled_text.style.styles.push((
                                font_size..renderer.styled_text.text.len(),
                                TextStyleProperty::FontSize(size),
                            ));
                        }
                        renderer.pop_bold();
                        // Chosen margin for headings
                        let margin = match level {
                            HeadingLevel::H1 => 40,
                            HeadingLevel::H2 => 30,
                            HeadingLevel::H3 => 25,
                            HeadingLevel::H4 => 20,
                            HeadingLevel::H5 => 15,
                            HeadingLevel::H6 => 10,
                        };
                        let text_input = TextInputElement::create(
                            renderer.elements,
                            renderer.gummy_tree,
                            renderer.access_tree,
                            renderer.by_internal_id,
                            "",
                        );
                        renderer.elements.get_mut(text_input).set_margin(
                            renderer.gummy_tree,
                            px(margin),
                            px(0),
                            px(margin),
                            px(0),
                        );
                        renderer
                            .elements
                            .get_mut(text_input)
                            .set_border_width_all(renderer.gummy_tree, px(0));
                        renderer
                            .elements
                            .get_as_mut::<TextInputElement>(text_input)
                            .disabled(true);
                        renderer.push_rich_text(Some(text_input));
                        renderer.font_size = None;
                    }
                    TagEnd::BlockQuote(_) => {}
                    TagEnd::CodeBlock =>
                    {
                        #[cfg(feature = "code_highlighting")]
                        if let Some(code_block_kind) = renderer.code_block_kind.take() {
                            let language = match code_block_kind {
                                pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                                pulldown_cmark::CodeBlockKind::Indented => "plaintext".to_string(),
                            };
                            let code_editor = CodeEditorElement::insert(
                                renderer.elements,
                                renderer.gummy_tree,
                                renderer.access_tree,
                                renderer.by_internal_id,
                                &renderer.styled_text.text,
                                &language,
                                "base16-ocean.dark",
                            );
                            renderer.push(code_editor);
                            renderer.styled_text = StyledText::new();
                        }
                    }
                    TagEnd::HtmlBlock => {}
                    TagEnd::List(_ordered) => {
                        renderer.pop_list_id();
                        renderer.pop_container();
                    }
                    TagEnd::Item => {
                        renderer.push_rich_text(None);
                        renderer.pop_container();
                    }
                    TagEnd::Emphasis => {
                        renderer.pop_italic();
                    }
                    TagEnd::Strong => {
                        renderer.pop_bold();
                    }
                    TagEnd::Strikethrough => {}
                    TagEnd::Superscript => {}
                    TagEnd::Subscript => {}
                    TagEnd::Link => {
                        renderer.pop_link();
                    }
                    TagEnd::Image => {
                        let text = &renderer.styled_text.text;
                        let text = TextElement::insert(
                            renderer.elements,
                            renderer.gummy_tree,
                            renderer.access_tree,
                            renderer.by_internal_id,
                            text,
                        );
                        renderer.push(text);
                        renderer.styled_text = StyledText::new();
                    }
                    TagEnd::MetadataBlock(_) => {}
                    _ => {}
                }
            }
            Event::Text(text) => {
                renderer.styled_text.text.push_str(&text);
            }
            Event::Code(text) => {
                let range = renderer.styled_text.text.len()..renderer.styled_text.text.len() + text.len();
                let font_family = "monospace";
                renderer
                    .styled_text
                    .style
                    .styles
                    .push((range.clone(), TextStyleProperty::FontFamily(font_family.to_string())));
                renderer
                    .styled_text
                    .style
                    .styles
                    .push((range.clone(), TextStyleProperty::FontSize(14.0)));
                renderer
                    .styled_text
                    .style
                    .styles
                    .push((range.clone(), TextStyleProperty::FontWeight(FontWeight::NORMAL)));
                let byte_range = renderer.styled_text.text.len()..renderer.styled_text.text.len() + text.len();
                renderer.styled_text.style.styles.push((
                    byte_range,
                    TextStyleProperty::BackgroundBrush(Brush::Color(rgb(0x2e, 0x2e, 0x2e))),
                ));
                renderer
                    .styled_text
                    .style
                    .styles
                    .push((range.clone(), TextStyleProperty::Color(Brush::Color(Color::WHITE))));
                renderer.styled_text.text.push_str(&text);
            }
            Event::SoftBreak => {
                renderer.push_text(" ");
            }
            Event::HardBreak => {
                renderer.push_text("\n");
            }
            Event::Rule => {
                let rule = ContainerElement::create(
                    renderer.elements,
                    renderer.gummy_tree,
                    renderer.access_tree,
                    renderer.by_internal_id,
                );
                renderer
                    .elements
                    .get_mut(rule)
                    .set_display(renderer.gummy_tree, Display::Block);
                renderer.elements.get_mut(rule).set_width(renderer.gummy_tree, pct(100));
                renderer.elements.get_mut(rule).set_height(renderer.gummy_tree, px(1));
                renderer
                    .elements
                    .get_mut(rule)
                    .set_background_brush(Brush::Color(rgb(0xD3, 0xD3, 0xD3)));
                renderer
                    .elements
                    .get_mut(rule)
                    .set_margin(renderer.gummy_tree, px(20), px(0), px(20), px(0));
                renderer.push(rule);
            }
            _ => {}
        }
    }

    renderer.element_stack.remove(0)
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[test]
    fn markdown_builds_nested_content_and_queues_images() {
        let mut app = App::new();
        let root = render_markdown(
            &mut app,
            "# Heading\n\nPlain **bold** text.\n\n1. first\n2. second\n\n![alt](image.png)\n\n```rust\nfn main() {}\n```\n\n---",
        );
        let mut pending = vec![root];
        let mut text = Vec::new();
        let mut code_editors = 0;
        while let Some(handle) = pending.pop() {
            let element = app.elements.get(handle);
            pending.extend(element.element_data().children.iter().copied());
            let element = element as &dyn Any;
            if let Some(input) = element.downcast_ref::<TextInputElement>() {
                text.push(input.state.editor().text().into_iter().collect::<String>());
            }
            if element.is::<CodeEditorElement>() {
                code_editors += 1;
            }
        }

        for expected in ["Heading", "Plain bold text.", "1. first", "2. second", "fn main() {}\n"] {
            assert!(text.iter().any(|text| text == expected), "missing {expected:?}");
        }
        assert_eq!(code_editors, 1);
        assert!(
            app.pending_resources
                .contains(&(ResourceId::File(PathBuf::from("image.png")), ResourceType::Image,))
        );
    }
}
