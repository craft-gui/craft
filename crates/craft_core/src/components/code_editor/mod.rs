use crate::components::Context;
use crate::components::{Component, ComponentSpecification};
use crate::elements::{ElementStyles, TextInput};
use crate::events::CraftMessage;
use crate::events::CraftMessage::TextInputChanged;
use crate::events::Message;
use crate::style::FontStyle;
use crate::style::TextStyleProperty::{FontStyle as PropFontStyle, FontWeight, UnderlineSize};
use crate::style::{TextStyleProperty, Weight};
use crate::text::RangedStyles;
use crate::{Color};
use std::cell::RefCell;
use std::rc::Rc;
use syntect::easy::HighlightLines;
use syntect::parsing::{SyntaxSet};
use syntect::util::LinesWithEndings;

pub use syntect;
use syntect::dumps::from_reader;
use syntect::highlighting::ThemeSet;

const DEFAULT_SYNTAX_PACK: &[u8] = include_bytes!("../../../../syntect_dumper/pack.dump");
const DEFAULT_THEME_PACK: &[u8] = include_bytes!("../../../../syntect_dumper/theme_pack.dump");

thread_local! {
    static SYNTAX_THEME_CACHE: RefCell<Option<(SyntaxSet, Rc<ThemeSet>)>> = const { RefCell::new(None) };
}

fn get_syntax_and_theme() -> (SyntaxSet, Rc<ThemeSet>) {
    SYNTAX_THEME_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((ref ss, ref ts)) = *cache {
            return (ss.clone(), Rc::clone(ts));
        }

        let syntax_set: SyntaxSet = from_reader(DEFAULT_SYNTAX_PACK).expect("Failed to load syntax pack");

        let theme_set: ThemeSet = from_reader(DEFAULT_THEME_PACK).expect("Failed to load theme pack");
        let theme_set = Rc::new(theme_set);

        *cache = Some((syntax_set.clone(), Rc::clone(&theme_set)));
        (syntax_set, theme_set)
    })
}

pub struct CodeEditor {
    pub(crate) style: CodeEditorStyle,
    pub(crate) syntax_set: Option<SyntaxSet>,
    pub(crate) theme_set: Option<ThemeSet>,
    pub(crate) theme: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        CodeEditor {
            style: CodeEditorStyle::default(),
            syntax_set: None,
            theme_set: None,
            theme: "base16-ocean.dark".to_string(),
        }
    }
}

impl CodeEditor {
    pub fn new(style: CodeEditorStyle, syntax_set: SyntaxSet, theme_set: ThemeSet, theme: &str) -> Self {
        CodeEditor {
            style,
            syntax_set: Some(syntax_set),
            theme_set: Some(theme_set),
            theme: theme.to_string(),
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

pub struct CodeEditorStyle {
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
    code: &str,
    syntax_set: Option<&SyntaxSet>,
    theme_set: Option<&ThemeSet>,
    extension: &str,
    theme: &str,
) -> CodeEditorStyle {
    let (default_syntax_set, default_themes_set) = get_syntax_and_theme();
    let syntax_set = if let Some(syntax_set) = syntax_set { syntax_set } else { &default_syntax_set };

    let theme_set = if let Some(theme_set) = theme_set { theme_set } else { &default_themes_set };

    let syntax = syntax_set.find_syntax_by_extension(extension).unwrap_or(syntax_set.find_syntax_plain_text());

    let theme = &theme_set.themes[theme];

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

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let code = &context.props().text;

        TextInput::new(code)
            .margin(20, 20, 20, 0)
            .ranged_styles(context.state().style.ranged_styles.clone())
            .background(context.state().style.background_color)
            .color(context.state().style.foreground_color)
            .component()
    }

    fn update(context: &mut Context<Self>) {
        if let Message::CraftMessage(TextInputChanged(text)) = context.message() {
            context.state_mut().style = compute_code_editor_style(
                text,
                context.state().syntax_set.as_ref(),
                context.state().theme_set.as_ref(),
                &context.props().extension,
                context.state().theme.as_str(),
            );
        }

        if let Message::CraftMessage(CraftMessage::Initialized) = context.message() {
            context.state_mut().style = compute_code_editor_style(
                &context.props().text,
                context.state().syntax_set.as_ref(),
                context.state().theme_set.as_ref(),
                &context.props().extension,
                context.state().theme.as_str(),
            );
        }
    }
}
