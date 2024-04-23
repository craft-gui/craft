use crate::renderer::color::Color;

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Px(f32),
    Auto,
}

impl Unit {
    pub fn is_auto(&self) -> bool {
        matches!(self, Unit::Auto)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Display {
    Flex,
    Block,
    Grid,
}

#[derive(Clone, Copy, Debug)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Clone, Copy, Debug)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

pub type JustifyContent = AlignContent;

#[derive(Clone, Copy, Debug)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    pub width: Unit,
    pub height: Unit,
    pub x: f32,
    pub y: f32,
    pub display: Display,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub flex_direction: FlexDirection,

    pub color: Color,
    pub background: Color,
    pub font_size: f32,
}

fn unit_to_taffy_dimension(unit: Unit) -> taffy::Dimension {
    match unit {
        Unit::Px(px) => taffy::Dimension::Length(px),
        Unit::Auto => taffy::Dimension::Auto,
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            margin: [0.0; 4],
            padding: [0.0; 4],
            width: Unit::Auto,
            height: Unit::Auto,
            x: 0.0,
            y: 0.0,
            display: Display::Flex,
            align_items: None,
            justify_content: None,
            flex_direction: FlexDirection::Row,
            color: Color::new_from_rgba_u8(0, 0, 0, 255),
            background: Color::new_from_rgba_u8(0, 0, 0, 0),
            font_size: 16.0,
        }
    }
}

impl From<Style> for taffy::Style {
    fn from(style: Style) -> Self {
        let display = match style.display {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::Grid => taffy::Display::Grid,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension(style.width),
            height: unit_to_taffy_dimension(style.height),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: taffy::LengthPercentageAuto::Length(style.margin[3]),
            right: taffy::LengthPercentageAuto::Length(style.margin[1]),
            top: taffy::LengthPercentageAuto::Length(style.margin[0]),
            bottom: taffy::LengthPercentageAuto::Length(style.margin[2]),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: taffy::LengthPercentage::Length(style.padding[3]),
            right: taffy::LengthPercentage::Length(style.padding[1]),
            top: taffy::LengthPercentage::Length(style.padding[0]),
            bottom: taffy::LengthPercentage::Length(style.padding[2]),
        };

        let align_items = match style.align_items {
            None => None,
            Some(AlignItems::Start) => Some(taffy::AlignItems::Start),
            Some(AlignItems::End) => Some(taffy::AlignItems::End),
            Some(AlignItems::FlexStart) => Some(taffy::AlignItems::FlexStart),
            Some(AlignItems::FlexEnd) => Some(taffy::AlignItems::FlexEnd),
            Some(AlignItems::Center) => Some(taffy::AlignItems::Center),
            Some(AlignItems::Baseline) => Some(taffy::AlignItems::Baseline),
            Some(AlignItems::Stretch) => Some(taffy::AlignItems::Stretch),
        };

        let justify_content = match style.justify_content {
            None => None,
            Some(JustifyContent::Start) => Some(taffy::JustifyContent::Start),
            Some(JustifyContent::End) => Some(taffy::JustifyContent::End),
            Some(JustifyContent::FlexStart) => Some(taffy::JustifyContent::FlexStart),
            Some(JustifyContent::FlexEnd) => Some(taffy::JustifyContent::FlexEnd),
            Some(JustifyContent::Center) => Some(taffy::JustifyContent::Center),
            Some(JustifyContent::Stretch) => Some(taffy::JustifyContent::Stretch),
            Some(JustifyContent::SpaceBetween) => Some(taffy::JustifyContent::SpaceBetween),
            Some(JustifyContent::SpaceEvenly) => Some(taffy::JustifyContent::SpaceEvenly),
            Some(JustifyContent::SpaceAround) => Some(taffy::JustifyContent::SpaceAround),
        };

        let flex_direction = match style.flex_direction {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        };

        taffy::Style {
            size,
            flex_direction,
            margin,
            padding,
            justify_content,
            align_items,
            display,
            ..Default::default()
        }
    }
}
