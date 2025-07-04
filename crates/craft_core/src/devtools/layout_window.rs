use crate::components::{Component, ComponentSpecification, Context};
use crate::devtools::dev_tools_colors::{BORDER_COLOR, FIELD_NAME_COLOR, FIELD_VALUE_COLOR, ROW_BACKGROUND_COLOR};
use crate::elements::element::Element;
use crate::elements::{Container, ElementStyles, Text, TextInput};
use crate::events::{CraftMessage, Message};
use crate::geometry::side::Side;
use crate::style::style_flags::StyleFlags;
use crate::style::Display::Flex;
use crate::style::{Display, FlexDirection, Unit};
use crate::{palette, Color};
use taffy::Overflow;

fn format_option<T: std::fmt::Debug>(option: Option<T>) -> String {
    option.map_or("None".to_string(), |value| format!("{value:?}"))
}

fn field_row(
    field_name: &str,
    field_name_color: Color,
    field_value: &str,
    field_value_color: Color,
) -> ComponentSpecification {
    Container::new()
        .push(Text::new(field_name.to_lowercase().as_str()).color(field_name_color))
        .push(Text::new(field_value.to_lowercase().as_str()).color(field_value_color))
        .padding("0px", "10px", "0px", "10px")
        .component()
}


#[derive(Default)]
#[derive(PartialEq)]
pub(crate) enum LayoutTab {
    #[default]
    Styles,
    Computed
}

#[derive(Default)]
pub(crate) struct LayoutWindow {
    pub(crate) layout_tab: LayoutTab,
    pub(crate) style_search_query: String,
    pub(crate) computed_search_query: String,
}

#[derive(Default)]
pub(crate) struct LayoutWindowProps {
    pub(crate) selected_element: Option<Box<dyn Element>>,
}


fn tab_computed_styles(selected_element: &dyn Element, search: &str) -> Container {
    let computed_style = selected_element.layout_item();
    let mut computed_window = Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column);

    let box_model = &computed_style.computed_box_transformed;
    
    let rows = vec![
        ("Size", format!("({}px, {}px)", box_model.size.width, box_model.size.height)),
        ("Position", format!("({}, {})", box_model.position.x, box_model.position.y)),

        ("Padding Top", format!("{}px", box_model.padding.top)),
        ("Padding Right", format!("{}px", box_model.padding.right)),
        ("Padding Bottom", format!("{}px", box_model.padding.bottom)),
        ("Padding Left", format!("{}px", box_model.padding.left)),

        ("Margin Top", format!("{}px", box_model.margin.top)),
        ("Margin Right", format!("{}px", box_model.margin.right)),
        ("Margin Bottom", format!("{}px", box_model.margin.bottom)),
        ("Margin Left", format!("{}px", box_model.margin.left)),
        ("Border Top", {
            let s = computed_style.computed_border.get_side(Side::Top);
            format!("{}px, {}", s.width, s.color.to_rgba8())
        }),
        ("Border Right", {
            let s = computed_style.computed_border.get_side(Side::Right);
            format!("{}px, {}", s.width, s.color.to_rgba8())
        }),
        ("Border Bottom", {
            let s = computed_style.computed_border.get_side(Side::Bottom);
            format!("{}px, {}", s.width, s.color.to_rgba8())
        }),
        ("Border Left", {
            let s = computed_style.computed_border.get_side(Side::Left);
            format!("{}px, {}", s.width, s.color.to_rgba8())
        }),

        ("Content Size", format!("({}px, {}px)", computed_style.content_size.width, computed_style.content_size.height)),
        ("Scrollbar Size", format!("({}px, {}px)", computed_style.computed_scrollbar_size.width, computed_style.computed_scrollbar_size.height)),
        ("Scroll Thumb", format!("({}px, {}px)", computed_style.computed_scroll_thumb.width, computed_style.content_size.height)),
        ("Scroll Track", format!("({}px, {}px)", computed_style.computed_scroll_track.width, computed_style.content_size.height)),
        ("Max Scroll Y", computed_style.max_scroll_y.to_string()),
        ("Layout Order", computed_style.layout_order.to_string()),
    ];

    for (label, value) in rows.into_iter() {
        if label.to_lowercase().contains(&search.to_lowercase()) {
            computed_window = computed_window.push(field_row(
                &format!("{label}: "),
                FIELD_NAME_COLOR,
                &value,
                FIELD_VALUE_COLOR,
            ));
        }
    }

    computed_window
}


fn tab_styles(selected_element: &dyn Element, search: &str) -> Container {
    let style = selected_element.style();
    let mut fields = Vec::new();
    let search = search.to_ascii_lowercase();

    macro_rules! push_field {
        ($label:expr, $value:expr) => {
            fields.push((format!("{}: ", $label.to_string()), $value.to_string()));
        };
    }

    if style.dirty_flags.contains(StyleFlags::FONT_FAMILY) && let Some(family) = style.font_family().name() {
        push_field!("Font Family", family);
    }

    if style.dirty_flags.contains(StyleFlags::BOX_SIZING) {
        push_field!("Box Sizing", format!("{:?}", style.box_sizing()));
    }

    if style.dirty_flags.contains(StyleFlags::SCROLLBAR_WIDTH) {
        push_field!("Scrollbar Width", style.scrollbar_width().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::POSITION) {
        push_field!("Position", format!("{:?}", style.position()));
    }

    if style.dirty_flags.contains(StyleFlags::MARGIN) {
        let margin = style.margin();
        push_field!("Margin Top", margin.top.to_string());
        push_field!("Margin Right", margin.right.to_string());
        push_field!("Margin Bottom", margin.bottom.to_string());
        push_field!("Margin Left", margin.left.to_string());
    }

    if style.dirty_flags.contains(StyleFlags::PADDING) {
        let padding = style.padding();
        push_field!("Padding Top", padding.top.to_string());
        push_field!("Padding Right", padding.right.to_string());
        push_field!("Padding Bottom", padding.bottom.to_string());
        push_field!("Padding Left", padding.left.to_string());
    }

    if style.dirty_flags.contains(StyleFlags::GAP) {
        let gap = style.gap();
        push_field!("Row Gap", gap[0].to_string());
        push_field!("Column Gap", gap[1].to_string());
    }

    if style.dirty_flags.contains(StyleFlags::INSET) {
        let inset = style.inset();
        push_field!("Inset Top", inset.top.to_string());
        push_field!("Inset Right", inset.right.to_string());
        push_field!("Inset Bottom", inset.bottom.to_string());
        push_field!("Inset Left", inset.left.to_string());
    }

    if style.dirty_flags.contains(StyleFlags::WIDTH) {
        push_field!("Width", style.width().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::HEIGHT) {
        push_field!("Height", style.height().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::MAX_WIDTH) {
        push_field!("Max Width", style.max_width().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::MAX_HEIGHT) {
        push_field!("Max Height", style.max_height().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::MIN_WIDTH) {
        push_field!("Min Width", style.min_width().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::MIN_HEIGHT) {
        push_field!("Min Height", style.min_height().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::X) {
        push_field!("X", style.x().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::Y) {
        push_field!("Y", style.y().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::DISPLAY) {
        push_field!("Display", format!("{:?}", style.display()));
    }

    if style.dirty_flags.contains(StyleFlags::WRAP) {
        push_field!("Wrap", format!("{:?}", style.wrap()));
    }

    if style.dirty_flags.contains(StyleFlags::ALIGN_ITEMS) {
        push_field!("Align Items", format_option(style.align_items()));
    }

    if style.dirty_flags.contains(StyleFlags::JUSTIFY_CONTENT) {
        push_field!("Justify Content", format_option(style.justify_content()));
    }

    if style.dirty_flags.contains(StyleFlags::FLEX_DIRECTION) {
        push_field!("Flex Direction", format!("{:?}", style.flex_direction()));
    }

    if style.dirty_flags.contains(StyleFlags::FLEX_GROW) {
        push_field!("Flex Grow", style.flex_grow().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::FLEX_SHRINK) {
        push_field!("Flex Shrink", style.flex_shrink().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::FLEX_BASIS) {
        push_field!("Flex Basis", style.flex_basis().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::COLOR) {
        push_field!("Color", style.color().to_rgba8().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::BACKGROUND) {
        push_field!("Background", style.background().to_rgba8().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::FONT_SIZE) {
        push_field!("Font Size", style.font_size().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::FONT_WEIGHT) {
        push_field!("Font Weight", format!("{:?}", style.font_weight()));
    }

    if style.dirty_flags.contains(StyleFlags::FONT_STYLE) {
        push_field!("Font Style", format!("{:?}", style.font_style()));
    }

    if style.dirty_flags.contains(StyleFlags::OVERFLOW) {
        push_field!("Overflow", format!("{:?}", style.overflow()));
    }

    if style.dirty_flags.contains(StyleFlags::BORDER_COLOR) {
        push_field!("Border Color Top", style.border_color().top.to_rgba8().to_string());
        push_field!("Border Color Right", style.border_color().right.to_rgba8().to_string());
        push_field!("Border Color Bottom", style.border_color().bottom.to_rgba8().to_string());
        push_field!("Border Color Left", style.border_color().left.to_rgba8().to_string());
    }

    if style.dirty_flags.contains(StyleFlags::BORDER_WIDTH) {
        push_field!(
            "Border Width",
            style.border_width().to_array().map(|bw| bw.to_string()).join(", ")
        );
    }

    if style.dirty_flags.contains(StyleFlags::BORDER_RADIUS) {
        push_field!("Border Radius", format!("{:?}", style.border_radius()));
    }

    fields.into_iter().filter(|(label, _value)| {
        label.to_ascii_lowercase().contains(&search)
    }).fold(Container::new().display(Display::Flex).flex_direction(FlexDirection::Column), |acc, (label, value)| {
        acc.push(field_row(&label, FIELD_NAME_COLOR, &value, FIELD_VALUE_COLOR))
    })
}

impl Component for LayoutWindow {
    type GlobalState = ();
    type Props = LayoutWindowProps;
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let active_tab_color = palette::css::MEDIUM_AQUAMARINE;
        
        let mut styles_window = Container::new()
            .width(Unit::Percentage(100.0))
            .display(Flex)
            .flex_direction(FlexDirection::Column)
            .height("50%")
            .max_height("50%")
            .overflow(Overflow::Scroll)
            .background(ROW_BACKGROUND_COLOR)
            .push(Container::new().border_width("2px", "0px", "2px", "0px").border_color(BORDER_COLOR)
                .push(Text::new("Styles")
                          .color(if context.state().layout_tab == LayoutTab::Styles { active_tab_color} else { Color::from_rgb8(230, 230, 230) })
                          .padding("10px", "0px", "10px", "10px")
                          .id("tab_styles")
                )
                .push(Text::new("Computed")
                    .color(if context.state().layout_tab == LayoutTab::Computed { active_tab_color} else { Color::from_rgb8(230, 230, 230) })
                          .padding("10px", "0px", "10px", "10px")
                          .id("tab_computed")
                )
            )
            .component();

        let selected_element = context.props().selected_element.as_ref();

        if let Some(selected_element) = selected_element {
            let state = context.state();
            
            match context.state().layout_tab {
                
                LayoutTab::Styles => {
                    styles_window.push_in_place(
                        TextInput::new(state.style_search_query.as_str())
                            .use_text_value_on_update(true)
                            .margin("10px", "0px", "20px", "10px")
                            .background(Color::from_rgb8(50, 50, 50))
                            .border_radius(10.0, 10.0, 10.0, 10.0)
                            .color(Color::WHITE)
                            .max_width("200px").id("style_search_query")
                            .key("style_search_query")
                            .component()
                    );
                    styles_window.push_in_place(tab_styles(selected_element.as_ref(), context.state().style_search_query.as_str()).component())
                }
                LayoutTab::Computed => {
                    styles_window.push_in_place(
                        TextInput::new(state.computed_search_query.as_str())
                            .use_text_value_on_update(true)
                            .margin("10px", "0px", "20px", "10px")
                            .background(Color::from_rgb8(50, 50, 50))
                            .border_radius(10.0, 10.0, 10.0, 10.0)
                            .color(Color::WHITE)
                            .max_width("200px")
                            .id("computed_search_query")
                            .key("computed_search_query")
                            .component()
                    );
                    styles_window.push_in_place(tab_computed_styles(selected_element.as_ref(), context.state().computed_search_query.as_str()).component())
                }
            }
        }

        styles_window
    }

    fn update(context: &mut Context<Self>) {
        if let Some(id) = context.target().and_then(|e| e.get_id().clone()) {
            if context.message().clicked() {
                if id == "tab_styles" {
                    context.state_mut().layout_tab = LayoutTab::Styles
                } else if id == "tab_computed" {
                    context.state_mut().layout_tab = LayoutTab::Computed
                }
            }

            if let Message::CraftMessage(CraftMessage::TextInputChanged(text)) = context.message() {
                if id == "computed_search_query" {
                    context.state_mut().computed_search_query = text.to_string();
                } else if id == "style_search_query" {
                    context.state_mut().style_search_query = text.to_string();
                }
            }
        }
    }
}