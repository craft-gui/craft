use std::fmt::{Display, Formatter};

pub struct OkuOptions {
    pub renderer: RendererType,
    pub window_title: String,
}

impl Default for OkuOptions {
    fn default() -> Self {
        Self {
            renderer: RendererType::Wgpu,
            window_title: "oku".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default, Copy, Clone, Debug)]
pub enum RendererType {
    Software,
    #[default]
    Wgpu,
}

#[cfg(target_os = "android")]
#[derive(Default, Copy, Clone, Debug)]
pub enum RendererType {
    #[default]
    Wgpu,
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
#[derive(Default, Copy, Clone, Debug)]
pub enum RendererType {
    #[cfg(not(target_os = "android"))]
    Software,
    Wgpu,
    #[default]
    Vello,
}

impl Display for RendererType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_os = "android"))]
            RendererType::Software => write!(f, "software(tiny-skia)"),
            RendererType::Wgpu => write!(f, "wgpu"),
            RendererType::Vello => write!(f, "vello/wgpu")
        }
    }
}
