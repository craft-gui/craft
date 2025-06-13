use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::{ElementStyles, TextInput};
use craft::events::CraftMessage::TextInputChanged;
use craft::events::{CraftMessage, Message};
use craft::style::FontStyle;
use craft::style::TextStyleProperty::{FontStyle as PropFontStyle, FontWeight, UnderlineSize};
use craft::style::{TextStyleProperty, Weight};
use craft::text::RangedStyles;
use craft::{Color, WindowContext};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct CodeEditor {
    pub(crate) style: CodeEditorStyle,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: syntect::highlighting::ThemeSet,
}

impl Default for CodeEditor {
    fn default() -> CodeEditor {
        CodeEditor {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
            style: CodeEditorStyle::default(),
        }
    }
}

#[derive(Default)]
pub struct CodeEditorProps {
    pub(crate) text: String
}

fn syntect_color_to_color(color: syntect::highlighting::Color) -> Color {
    Color::from_rgba8(color.r, color.g, color.b, color.a)
}

pub(crate) struct CodeEditorStyle {
    pub(crate) ranged_styles: RangedStyles,
    pub(crate) foreground_color: Color,
    pub(crate) background_color: Color,
}

impl Default for CodeEditorStyle {
    fn default() -> Self {
        Self {
            ranged_styles: Default::default(),
            foreground_color: Color::WHITE,
            background_color: Color::BLACK,
        }
    }
}

fn compute_code_editor_style(code: &String, syntax_set: &SyntaxSet, theme_set: &syntect::highlighting::ThemeSet) -> CodeEditorStyle {
    let syntax = syntax_set.find_syntax_by_extension("rs").unwrap();
    let theme = &theme_set.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut ranged_styles = RangedStyles::default();
    let mut global_offset = 0;
    for line in LinesWithEndings::from(code) {
        let styled = highlighter.highlight_line(line, syntax_set).unwrap();

        let mut local_offset = 0;
        for (style, text) in styled {
            let byte_len = text.len();
            if byte_len == 0 {
                continue;
            }

            let start = global_offset + local_offset;
            let end = start + byte_len;
            let range = start..end;
            
            if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                ranged_styles.styles.push((range.clone(), FontWeight(Weight::BOLD)));
            }
            if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                ranged_styles.styles.push((range.clone(), PropFontStyle(FontStyle::Italic)));
            }
            if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
                ranged_styles.styles.push((range.clone(), UnderlineSize(1.0)));
            }

            ranged_styles.styles.push((range, TextStyleProperty::Color(syntect_color_to_color(style.foreground))));

            local_offset += byte_len;
        }

        global_offset += line.len();
    }

    let background_color = if let Some(bg_color) = theme.settings.background {
        syntect_color_to_color(bg_color)
    } else {
        Color::BLACK
    };
    
    let foreground_color = if let Some(foreground_color) = theme.settings.foreground {
        syntect_color_to_color(foreground_color)
    } else {
        Color::WHITE
    };
    
    CodeEditorStyle {
        ranged_styles,
        foreground_color,
        background_color,
    }
}

impl Component for CodeEditor {
    type GlobalState = ();
    type Props = CodeEditorProps;
    type Message = ();
    
    fn view(&self, _global_state: &Self::GlobalState, props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window: &WindowContext) -> ComponentSpecification {
        let code = &props.text;
        
        TextInput::new(code)
            .ranged_styles(self.style.ranged_styles.clone())
            .background(self.style.background_color)
            .color(self.style.foreground_color)
            .component()
    }

    fn update(&mut self, _global_state: &mut Self::GlobalState, props: &Self::Props, _event: &mut Event, message: &Message) {
        if let Message::CraftMessage(TextInputChanged(text)) = message {
            self.style = compute_code_editor_style(text, &self.syntax_set, &self.theme_set);
        }
        
        if let Message::CraftMessage(CraftMessage::Initialized) = message {
            self.style = compute_code_editor_style(&props.text, &self.syntax_set, &self.theme_set);
        }
    }
    
}