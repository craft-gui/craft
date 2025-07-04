use crate::style::FontStyle;
use crate::Color;
use crate::style::Weight;
use std::path::PathBuf;
use std::str::FromStr;
use crate::components::{CodeEditor, CodeEditorProps};
use crate::components::{Component, ComponentSpecification, Props};
use crate::elements::Container;
use crate::elements::{ElementStyles, Image, Text, TextInput};
use craft_resource_manager::ResourceIdentifier;
use crate::rgb;
use crate::style::{Display, FlexDirection, TextStyleProperty, Unit};
use crate::text::RangedStyles;
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

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

struct MarkdownRenderer<'a> {
    element_stack: Vec<ComponentSpecification>,
    list_ids: Vec<Option<u64>>,
    styled_text: StyledText,
    bold: Option<usize>,
    font_size: Option<usize>,
    italic: Option<usize>,
    link: Option<(usize, String)>,
    code_block_kind: Option<pulldown_cmark::CodeBlockKind<'a>>,
}

impl<'a> MarkdownRenderer<'a> {
    pub fn new() -> Self {
        MarkdownRenderer {
            element_stack: vec![Container::new().display(Display::Block).component()],
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

    pub fn push(&mut self, component_specification: ComponentSpecification) {
        self.current_element().push_in_place(component_specification);
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

    pub fn push_container(&mut self, container: ComponentSpecification) {
        self.element_stack.push(container);
    }

    pub fn pop_container(&mut self) {
        let container = self.element_stack.pop().expect("Element stack should not be empty");
        self.push(container);
    }

    pub fn current_element(&mut self) -> &mut ComponentSpecification {
        self.element_stack.last_mut().expect("Element stack should not be empty")
    }

    pub fn push_text(&mut self, text: &str) {
        self.styled_text.text.push_str(text);
    }

    pub fn push_rich_text(&mut self, text_input: Option<TextInput>) {
        if self.styled_text.text.is_empty() {
            return;
        }

        let mut text = if let Some(text_input) = text_input {
            let mut text_input = text_input;
            text_input.text = Some(self.styled_text.text.clone());
            text_input
        } else {
            TextInput::new(&self.styled_text.text)
                .display(Display::Block)
                .border_width(0, 0, 0, 0)
                .disable()
        };

        text.ranged_styles = Some(self.styled_text.style.clone());
        self.push(text.component());
        self.styled_text = StyledText::new();
    }

    pub fn push_link(&mut self, url: String) {
        self.link = Some((self.styled_text.text.len(), url));
    }

    pub fn pop_link(&mut self) {
        if let Some((link_start, link)) = &self.link {
            let end = self.styled_text.text.len();
            self.styled_text.style.styles.push(
                (*link_start..end, TextStyleProperty::Link(link.clone())),
            );
            self.styled_text.style.styles.push(
                (*link_start..end, TextStyleProperty::Color(rgb(0, 0, 238))),
            );
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
            self.styled_text.style.styles.push(
                (bold_start..end, TextStyleProperty::FontWeight(Weight::BOLD)),
            );
            self.bold = None;
        }
    }

    pub fn pop_italic(&mut self) {
        if let Some(start) = self.italic {
            let end = self.styled_text.text.len();
            self.styled_text.style.styles.push(
                (start..end, TextStyleProperty::FontStyle(FontStyle::Italic)),
            );
            self.italic = None;
        }
    }
}

pub fn render_markdown(markdown: &str, ) -> ComponentSpecification {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut renderer = MarkdownRenderer::new();

    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
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
                        let padding = if children_count == 0 {
                            0
                        } else {
                            20
                        };
                        renderer.push_container(Container::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .margin(0, 0, 0, padding)
                            .component())
                    }
                    Tag::Item => {
                        if let Some(id) = renderer.list_id() {
                            let offset = renderer.current_element().children.len() as u64;
                            renderer.push_text(&format!("{}. ", id + offset));
                        } else {
                            renderer.push_text("• ");
                        }
                        let item_container = Container::new()
                            .display(Display::Block)
                            .border_width(0, 0, 0, 0)
                            .component();
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
                    Tag::Link {dest_url, .. } => {
                        renderer.push_link(dest_url.to_string());
                    }
                    Tag::Image { dest_url, .. } => {
                        let resource = if dest_url.starts_with("http") {
                            ResourceIdentifier::Url(dest_url.to_string())
                        } else {
                            ResourceIdentifier::File(PathBuf::from_str(&dest_url).expect("Invalid file path for image"))
                        };
                        renderer.push(
                            Container::new().push(
                                Image::new(resource).width(Unit::Auto).height(Unit::Auto)).component()
                        )
                    }
                    _ => {  }
                }
            }
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
                            renderer.styled_text.style.styles.push(
                                (font_size..renderer.styled_text.text.len(), TextStyleProperty::FontSize(size)),
                            );
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
                        let text_input = TextInput::new("")
                            .margin(margin, 0, margin, 0)
                            .border_width(0, 0 , 0, 0)
                            .disable();
                        renderer.push_rich_text(Some(text_input));
                        renderer.font_size = None;
                    }
                    TagEnd::BlockQuote(_) => {}
                    TagEnd::CodeBlock => {
                        if let Some(code_block_kind) = renderer.code_block_kind.take() {
                            let language = match code_block_kind {
                                pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                                pulldown_cmark::CodeBlockKind::Indented => "plaintext".to_string(),
                            };
                            let code_editor = CodeEditor::component().props(Props::new(CodeEditorProps {
                                text: renderer.styled_text.text.clone(),
                                extension: language,
                            }));
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
                        let text = Text::new(text);
                        renderer.push(text.component());
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
                renderer.styled_text.style.styles.push(
                    (range.clone(), TextStyleProperty::FontFamily(font_family.to_string())),
                );
                renderer.styled_text.style.styles.push(
                    (range.clone(), TextStyleProperty::FontSize(14.0)),
                );
                renderer.styled_text.style.styles.push(
                    (range.clone(), TextStyleProperty::FontWeight(Weight::NORMAL)),
                );
                let byte_range = renderer.styled_text.text.len()..renderer.styled_text.text.len() + text.len();
                renderer.styled_text.style.styles.push(
                    (byte_range, TextStyleProperty::BackgroundColor(rgb(0x2e, 0x2e, 0x2e))),
                );
                renderer.styled_text.style.styles.push(
                    (range.clone(), TextStyleProperty::Color(Color::WHITE)),
                );
                renderer.styled_text.text.push_str(&text);
            }
            Event::SoftBreak => {
                renderer.push_text(" ");
            }
            Event::HardBreak => {
                renderer.push_text("\n");
            }
            Event::Rule => {
                let rule = Container::new()
                    .display(Display::Block)
                    .border_width(0, 0, 1, 0)
                    .border_color(rgb(0xD3, 0xD3, 0xD3))
                    .margin(20, 0, 20, 0)
                    .component();
                renderer.push(rule);
            },
            _ => { }
        }
    }

    renderer.element_stack.remove(0)
}