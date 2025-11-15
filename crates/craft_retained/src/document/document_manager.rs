use std::collections::HashMap;
use winit::window::WindowId;
use crate::app::{CURRENT_WINDOW_ID};
use crate::document::Document;

/// A wrapper to get a document with a window id or the current document.
pub struct DocumentManager {
    documents: HashMap<WindowId, Document>,
}

impl DocumentManager {
    pub(crate) fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub(crate) fn add_document(&mut self, window_id: WindowId, document: Document) {
        self.documents.insert(window_id, document);
    }

    pub fn get_current_document(&mut self) -> &mut Document {
        let window_id_key = &CURRENT_WINDOW_ID.get().unwrap();
        self.documents.get_mut(window_id_key).unwrap()
    }
    pub fn get_document_by_window_id(&mut self, window_id: WindowId) -> Option<&mut Document> {
        self.documents.get_mut(&window_id)
    }
}