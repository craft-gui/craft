pub(crate) mod component;
mod props;
mod update_result;

#[cfg(feature = "code_highlighting")]
mod code_editor;
#[cfg(feature = "link")]
mod web_link;

pub use crate::events::UserMessage;
pub use component::Component;
pub use component::Context;
pub use component::ComponentId;
pub use component::ComponentOrElement;
pub use component::ComponentSpecification;
pub use props::Props;
pub use update_result::Event;
pub use update_result::ImeAction;
pub use update_result::FocusAction;
pub use update_result::PointerCapture;

#[cfg(feature = "code_highlighting")]
pub use {
    code_editor::CodeEditor,
    code_editor::CodeEditorProps,
    code_editor::syntect,
};


#[cfg(feature = "link")]
pub use {
    web_link::WebLink,
    web_link::WebLinkProps,
    web_link::open,
};