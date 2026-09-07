use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::{FxHashMap, FxHashSet};

use slotmap::{DefaultKey, Key, SlotMap};

use crate::App;
use crate::accessibility::RetGuiAccessTree;
use crate::elements::{DynElement, ElementIds, ElementInternals, WindowElement};
use crate::events::EventKind;
use crate::layout::GummyTree;

static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(1);

/// Retained elements, kept separate from layout so both can be borrowed mutably.
pub struct RetainedElements {
    id: u64,
    slots: SlotMap<DefaultKey, Option<Box<dyn ElementInternals>>>,
}

impl RetainedElements {
    pub fn get(&self, element: DynElement) -> &dyn ElementInternals {
        assert_eq!(
            element.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        self.slots
            .get(element.key())
            .and_then(Option::as_deref)
            .expect("element handle no longer belongs to this store")
    }

    pub fn contains(&self, element: DynElement) -> bool {
        element.store_id() == self.id && self.slots.get(element.key()).is_some_and(Option::is_some)
    }

    pub fn get_mut(&mut self, element: DynElement) -> &mut dyn ElementInternals {
        assert_eq!(
            element.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        self.slots
            .get_mut(element.key())
            .and_then(Option::as_deref_mut)
            .expect("element handle no longer belongs to this store")
    }

    /// Returns a retained element mutably, or `None` when the handle is stale
    /// or belongs to another store.
    pub fn try_get_mut(&mut self, element: DynElement) -> Option<&mut dyn ElementInternals> {
        if element.store_id() != self.id {
            return None;
        }
        self.slots.get_mut(element.key()).and_then(Option::as_deref_mut)
    }

    pub fn get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> &mut T {
        (self.get_mut(element) as &mut dyn Any)
            .downcast_mut()
            .expect("typed element handle changed type")
    }

    pub fn try_get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> Option<&mut T> {
        Some(
            (self.try_get_mut(element)? as &mut dyn Any)
                .downcast_mut()
                .expect("typed element handle changed type"),
        )
    }

    pub(crate) fn new() -> Self {
        Self {
            id: NEXT_STORE_ID.fetch_add(1, Ordering::Relaxed),
            slots: SlotMap::with_key(),
        }
    }

    pub fn insert_with(
        &mut self,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut FxHashMap<u64, DynElement>,
        create: impl FnOnce(DynElement, RetGuiAccessTree) -> Box<dyn ElementInternals>,
    ) -> DynElement {
        let access_tree = access_tree.clone();
        let store_id = self.id;
        let key = self
            .slots
            .insert_with_key(|key| Some(create(DynElement::from_key(key, store_id), access_tree)));
        let handle = DynElement::from_key(key, store_id);
        let id = self.get(handle).element_data().internal_id;
        by_internal_id.insert(id, handle);
        handle
    }

    pub fn try_get(&self, element: DynElement) -> Option<&dyn ElementInternals> {
        if element.store_id() != self.id {
            return None;
        }
        self.slots.get(element.key()).and_then(Option::as_deref)
    }

    /// Fast retained-tree lookup for handles already validated when they were
    /// attached to this store.
    pub fn get_for_draw(&self, element: DynElement) -> &dyn ElementInternals {
        debug_assert_eq!(element.store_id(), self.id);
        self.slots[element.key()]
            .as_deref()
            .expect("retained tree contains a deleted element")
    }

    pub fn get_as<T: ElementInternals>(&self, element: DynElement) -> &T {
        (self.get(element) as &dyn Any)
            .downcast_ref()
            .expect("typed element handle changed type")
    }

    pub fn try_get_as<T: ElementInternals>(&self, element: DynElement) -> Option<&T> {
        Some(
            (self.try_get(element)? as &dyn Any)
                .downcast_ref()
                .expect("typed element handle changed type"),
        )
    }

    pub(crate) fn delete_all_children(
        &mut self,
        gummy_tree: &mut GummyTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        parent: DynElement,
    ) {
        let roots = mem::take(&mut self.get_mut(parent).element_data_mut().children);
        if roots.is_empty() {
            return;
        }

        for root in &roots {
            self.get_mut(*root).element_data_mut().parent = None;
        }

        let mut subtree = Vec::new();
        let mut seen = FxHashSet::default();
        let mut pending = roots.clone();
        while let Some(element) = pending.pop() {
            if !seen.insert(element) {
                continue;
            }
            pending.extend(self.get(element).element_data().children.iter().copied());
            subtree.push(element);
        }

        if let Some(focused) = *focus
            && seen.contains(&focused)
        {
            self.get_mut(focused).unfocus(event_queue, focus);
        }

        if let Some(window) = self.get(parent).element_data().window {
            let capture = &mut self.get_as_mut::<WindowElement>(window).pointer_capture;
            for element in &subtree {
                capture.remove_element(*element);
            }
        }

        let layout_parent = self.get(parent).child_layout_parent();
        let layout_roots = roots
            .iter()
            .filter_map(|root| self.get(*root).element_data().layout.gummy_node_id)
            .collect::<Vec<_>>();
        if let Some(parent) = layout_parent {
            gummy_tree.set_children(parent, &[]);
        }
        for root in layout_roots {
            gummy_tree.remove_subtree(root);
        }

        let parent_data = self.get(parent).element_data();
        if let Some(parent_key) = parent_data.access_key {
            parent_data.access_tree.set_children(parent_key, &[]);
        }

        for handle in subtree.into_iter().rev() {
            if let Some(element) = self.slots.remove(handle.key()).flatten() {
                by_internal_id.remove(&element.element_data().internal_id);
            }
        }

        self.get(parent).request_window_redraw();
    }

    /// Mutates one element while retaining exclusive access to the rest of the
    /// store. This is used by tree algorithms that recurse through handles.
    pub fn dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Self) -> R,
    ) -> R {
        assert_eq!(
            handle.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        let mut element = self.slots[handle.key()]
            .take()
            .expect("element is already being visited");
        let result = callback(element.as_mut(), self);
        self.slots[handle.key()] = Some(element);
        result
    }

    /// Mutates a retained element and returns `None` when its handle is stale or
    /// belongs to another store.
    pub fn try_dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Self) -> R,
    ) -> Option<R> {
        if handle.store_id() != self.id {
            return None;
        }
        let mut element = self.slots.get_mut(handle.key())?.take()?;
        let result = callback(element.as_mut(), self);
        if let Some(slot) = self.slots.get_mut(handle.key()) {
            *slot = Some(element);
        }
        Some(result)
    }

    pub fn store_id(&self) -> u64 {
        self.id
    }
}

/// A copyable, type-safe handle to application state stored in [`App`].
///
/// Use [`State::update`] when a mutation should produce a value for a later UI
/// update. The callback only borrows `T`; after it returns, `App` is
/// available again for element mutation.
///
/// ```
/// use retgui::App;
///
/// let mut app = App::new();
/// let count = app.insert_state(0_i64);
/// let next = count.update(&mut app, |count| {
///     *count += 1;
///     *count
/// });
///
/// assert_eq!(next, 1);
/// assert_eq!(*count.read(&app), 1);
/// ```
pub struct State<T> {
    key: DefaultKey,
    store_id: u64,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.store_id == other.store_id
    }
}

impl<T> Eq for State<T> {}

impl<T> std::fmt::Debug for State<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("State")
            .field(&(self.store_id, self.key.data()))
            .finish()
    }
}

impl<T: 'static> State<T> {
    pub(crate) fn insert(states: &mut SlotMap<DefaultKey, Box<dyn Any>>, store_id: u64, value: T) -> Self {
        Self {
            key: states.insert(Box::new(value)),
            store_id,
            marker: PhantomData,
        }
    }

    pub fn read_from(self, states: &SlotMap<DefaultKey, Box<dyn Any>>, store_id: u64) -> &T {
        assert_eq!(self.store_id, store_id, "state handle belongs to a different store");
        states[self.key]
            .downcast_ref()
            .expect("state handle was used with the wrong store")
    }

    pub fn write_to(self, states: &mut SlotMap<DefaultKey, Box<dyn Any>>, store_id: u64) -> &mut T {
        assert_eq!(self.store_id, store_id, "state handle belongs to a different store");
        states[self.key]
            .downcast_mut()
            .expect("state handle was used with the wrong store")
    }

    /// Borrows this state value from its arena.
    pub fn read(self, app: &App) -> &T {
        app.state(self)
    }

    /// Mutably borrows this state value from its arena.
    pub fn write(self, app: &mut App) -> &mut T {
        app.state_mut(self)
    }

    /// Applies a scoped state mutation.
    ///
    /// The callback intentionally receives only the state value. Any UI work
    /// happens after it returns, when the exclusive state borrow has ended.
    pub fn update<R>(self, app: &mut App, callback: impl FnOnce(&mut T) -> R) -> R {
        callback(app.state_mut(self))
    }
}
