use std::sync::Arc;

pub(crate) struct CosmicFontBlobAdapter {
    font: Arc<cosmic_text::Font>,
}

/// Adapter to allow `cosmic_text::Font` to be used as a Blob.
impl CosmicFontBlobAdapter {
    pub(crate) fn new(font: Arc<cosmic_text::Font>) -> Self {
        Self { font }
    }
}

impl AsRef<[u8]> for CosmicFontBlobAdapter {
    fn as_ref(&self) -> &[u8] {
        self.font.data()
    }
}