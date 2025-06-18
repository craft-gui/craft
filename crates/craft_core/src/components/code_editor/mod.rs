use crate::events::Message;
use crate::components::{Component, ComponentId, ComponentSpecification, Event};
use crate::elements::{ElementStyles, TextInput};
use crate::events::CraftMessage::TextInputChanged;
use crate::events::{CraftMessage};
use crate::style::FontStyle;
use crate::style::TextStyleProperty::{FontStyle as PropFontStyle, FontWeight, UnderlineSize};
use crate::style::{TextStyleProperty, Weight};
use crate::text::RangedStyles;
use crate::{Color, WindowContext};
use syntect::easy::HighlightLines;
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::LinesWithEndings;

pub use syntect as syntect;
use syntect::highlighting::ThemeSet;

pub struct CodeEditor {
    pub(crate) style: CodeEditorStyle,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: ThemeSet,
}

impl CodeEditor {
    pub fn new(style: CodeEditorStyle, syntax_set: SyntaxSet, theme_set: ThemeSet) -> Self {
        CodeEditor {
            style,
            syntax_set,
            theme_set,
        }
    }
}

impl Default for CodeEditor {
    fn default() -> CodeEditor {
        let mut syntax_set_builder = SyntaxSetBuilder::new();
        let toml = SyntaxDefinition::load_from_str(include_str!("./TOML.sublime-syntax"), false, None).unwrap();
        let bash = SyntaxDefinition::load_from_str(include_str!("./Bash.sublime-syntax"), false, None).unwrap();
        let rust = SyntaxDefinition::load_from_str(include_str!("./Rust.sublime-syntax"), false, None).unwrap();
        syntax_set_builder.add(toml);
        syntax_set_builder.add(bash);
        syntax_set_builder.add(rust);
        syntax_set_builder.add_plain_text_syntax();
        CodeEditor {
            syntax_set: syntax_set_builder.build(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
            style: CodeEditorStyle::default(),
        }
    }
}

#[derive(Default)]
pub struct CodeEditorProps {
    pub(crate) text: String,
    pub extension: String,
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

fn compute_code_editor_style(
    code: &String,
    syntax_set: &SyntaxSet,
    theme_set: &syntect::highlighting::ThemeSet,
    extension: &str,
) -> CodeEditorStyle {
    let syntax = syntax_set.find_syntax_by_extension(extension).unwrap_or(syntax_set.find_syntax_plain_text());
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

    let background_color =
        if let Some(bg_color) = theme.settings.background { syntect_color_to_color(bg_color) } else { Color::BLACK };

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

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        let code = &props.text;

        TextInput::new(code)
            .margin(20, 20, 20, 0)
            .ranged_styles(self.style.ranged_styles.clone())
            .background(self.style.background_color)
            .color(self.style.foreground_color)
            .component()
    }

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        props: &Self::Props,
        _event: &mut Event,
        message: &Message,
    ) {
        if let Message::CraftMessage(TextInputChanged(text)) = message {
            self.style = compute_code_editor_style(text, &self.syntax_set, &self.theme_set, &props.extension);
        }

        if let Message::CraftMessage(CraftMessage::Initialized) = message {
            self.style = compute_code_editor_style(&props.text, &self.syntax_set, &self.theme_set, &props.extension);
        }
    }
}
