use accesskit::DeactivationHandler;

pub(crate) struct CraftDeactivationHandler {}

impl DeactivationHandler for CraftDeactivationHandler {
    fn deactivate_accessibility(&mut self) {}
}

impl CraftDeactivationHandler {
    pub fn new() -> Self {
        CraftDeactivationHandler {}
    }
}
