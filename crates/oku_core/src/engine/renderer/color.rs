#[derive(Clone, Default, Debug, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new_from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

    pub const fn new_from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap();
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r as u8, self.g as u8, self.b as u8, self.a as u8)
    }

    pub const fn r_u8(&self) -> u8 {
        self.r as u8
    }
    pub const fn g_u8(&self) -> u8 {
        self.g as u8
    }
    pub const fn b_u8(&self) -> u8 {
        self.b as u8
    }
    pub const fn a_u8(&self) -> u8 {
        self.a as u8
    }

    pub const RED: Color = Color::new_from_rgba_u8(255, 0, 0, 255);
    pub const GREEN: Color = Color::new_from_rgba_u8(0, 255, 0, 255);
    pub const BLUE: Color = Color::new_from_rgba_u8(0, 0, 255, 255);
    pub const WHITE: Color = Color::new_from_rgba_u8(255, 255, 255, 255);
    pub const BLACK: Color = Color::new_from_rgba_u8(0, 0, 0, 255);
}
