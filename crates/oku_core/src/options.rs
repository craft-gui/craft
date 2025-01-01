use std::fmt::{Display, Formatter};

pub struct OkuOptions {
    pub renderer: RendererType,
    pub window_title: String,
}

impl Default for OkuOptions {
    fn default() -> Self {
        Self {
            renderer: RendererType::default(),
            window_title: "oku".to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RendererType {
    #[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))]
    Software,
    #[cfg(feature = "wgpu_renderer")]
    Wgpu,
    #[cfg(feature = "vello_renderer")]
    Vello,
    Blank
}

impl Default for RendererType {
    fn default() -> Self {
        cfg_if::cfg_if!  {
            if #[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))] {
                RendererType::Software
            } else if #[cfg(feature = "wgpu_renderer")] {
                RendererType::Wgpu
            } else if #[cfg(feature = "vello_renderer")] {
                RendererType::Vello
            } else {
                RendererType::Blank
            }
        }
    }
}

impl Display for RendererType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))]
            RendererType::Software => write!(f, "software(tiny-skia)"),
            #[cfg(feature = "wgpu_renderer")]
            RendererType::Wgpu => write!(f, "wgpu"),
            #[cfg(feature = "vello_renderer")]
            RendererType::Vello => write!(f, "vello/wgpu"),
            RendererType::Blank => write!(f, "blank"),
        }
    }
}
