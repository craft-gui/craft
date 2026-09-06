use parley::{FontContext, TextStyle, TreeBuilder};

use retgui_primitives::brush::Brush;

pub(crate) fn create_font_context() -> FontContext {
    let mut context = FontContext::default();
    #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
    for bytes in [
        include_bytes!("../../../assets/fonts/Roboto-VariableFont_wdth,wght.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/Roboto-Italic-VariableFont_wdth,wght.ttf").as_slice(),
    ] {
        let fonts = context
            .collection
            .register_fonts(peniko::Blob::new(std::sync::Arc::new(bytes)), None);
        context
            .collection
            .append_generic_families(parley::GenericFamily::SystemUi, fonts.iter().map(|font| font.0));
    }
    // Uploads and text layout use separate contexts backed by the same collection.
    context.collection.make_shared();
    context
}

pub struct TextContext {
    pub font_context: FontContext,
    pub layout_context: parley::LayoutContext<Brush>,
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextContext {
    pub fn new() -> Self {
        Self {
            font_context: Default::default(),
            layout_context: Default::default(),
        }
    }

    pub fn tree_builder<'a>(&'a mut self, scale: f32, raw_style: &TextStyle<'_, '_, Brush>) -> TreeBuilder<'a, Brush> {
        self.layout_context
            .tree_builder(&mut self.font_context, scale, true, raw_style)
    }
}
