use retgui_resource_manager::ResourceError;

#[derive(Debug)]
pub enum RetGuiError {
    /// Thrown when an element cannot be found.
    ElementNotFound,
    /// A resource failed to load.
    ResourceError(ResourceError),
}

impl From<ResourceError> for RetGuiError {
    fn from(error: ResourceError) -> Self {
        Self::ResourceError(error)
    }
}

impl std::fmt::Display for RetGuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElementNotFound => f.write_str("Element not found"),
            Self::ResourceError(error) => write!(f, "Resource upload failed: {error}"),
        }
    }
}

impl std::error::Error for RetGuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ElementNotFound => None,
            Self::ResourceError(error) => Some(error),
        }
    }
}
