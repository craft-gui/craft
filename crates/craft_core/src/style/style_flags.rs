use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct StyleFlags: u128 {
        const FONT_FAMILY_LENGTH = 1 << 0;
        const FONT_FAMILY = 1 << 1;
        const BOX_SIZING = 1 << 2;
        const SCROLLBAR_WIDTH = 1 << 3;
        const POSITION = 1 << 4;
        const MARGIN = 1 << 5;
        const PADDING = 1 << 6;
        const GAP = 1 << 7;
        const INSET = 1 << 8;
        const WIDTH = 1 << 9;
        const HEIGHT = 1 << 10;
        const MAX_WIDTH = 1 << 11;
        const MAX_HEIGHT = 1 << 12;
        const MIN_WIDTH = 1 << 13;
        const MIN_HEIGHT = 1 << 14;
        const X = 1 << 15;
        const Y = 1 << 16;
        const DISPLAY = 1 << 17;
        const WRAP = 1 << 18;
        const ALIGN_ITEMS = 1 << 19;
        const JUSTIFY_CONTENT = 1 << 20;
        const FLEX_DIRECTION = 1 << 21;
        const FLEX_GROW = 1 << 22;
        const FLEX_SHRINK = 1 << 23;
        const FLEX_BASIS = 1 << 24;
        const COLOR = 1 << 25;
        const BACKGROUND = 1 << 26;
        const FONT_SIZE = 1 << 27;
        const FONT_WEIGHT = 1 << 28;
        const FONT_STYLE = 1 << 29;
        const OVERFLOW = 1 << 30;
        const BORDER_COLOR = 1 << 31;
        const BORDER_WIDTH = 1 << 32;
        const BORDER_RADIUS = 1 << 33;
        const SCROLLBAR_COLOR = 1 << 34;
        const SCROLLBAR_RADIUS = 1 << 35;
        const SCROLLBAR_THUMB_MARGIN = 1 << 36;
        const VISIBLE = 1 << 37;
        const UNDERLINE = 1 << 38;
    }
}
