use rustc_hash::FxHashSet;
use crate::components::ComponentId;

/// Tracks:
/// - Who read from global state.
/// - If anybody wrote to global state.
/// - Who wrote to a component's state.
#[derive(Default)]
#[derive(Debug)]
pub struct TrackedChanges {
    pub global_reads: FxHashSet<ComponentId>,
    pub writes: FxHashSet<ComponentId>,
    pub wrote_to_global_state: bool,
}