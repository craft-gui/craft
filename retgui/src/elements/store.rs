use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::{DefaultKey, Key, SlotMap};

use crate::accessibility::RetGuiAccessTree;
#[cfg(feature = "audio")]
use crate::elements::audio::AudioContext;
use crate::elements::element_data::ElementData;
use crate::elements::gui_actions::GuiActionQueue;
use crate::elements::{DynElement, ElementInternals};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::layout::layout_context::LayoutContext;
use crate::window_manager::WindowManager;
use retgui_resource_manager::ResourceId;
use retgui_resource_manager::resource_type::ResourceType;
use std::collections::VecDeque;

static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(1);

/// Owns every retained element and application state value.
pub struct Elements {
    elements: RetainedElements,
    states: SlotMap<DefaultKey, Box<dyn Any>>,
    by_internal_id: FxHashMap<u64, DynElement>,
    pub(crate) access_tree: RetGuiAccessTree,
    pub(crate) gummy_tree: GummyTree,
    pub(crate) window_manager: WindowManager,
    pub(crate) pending_resources: VecDeque<(ResourceId, ResourceType)>,
    pub(crate) event_queue: VecDeque<EventKind>,
    pending_animation_updates: Vec<(DynElement, bool)>,
    pub(crate) focus: Option<DynElement>,
    pub(crate) focus_outline_visible: bool,
    #[cfg(feature = "audio")]
    pub(crate) audio_context: Option<AudioContext>,
    gui_actions: GuiActionQueue,
}

/// Retained elements, kept separate from layout so both can be borrowed mutably.
pub(crate) struct RetainedElements {
    id: u64,
    slots: SlotMap<DefaultKey, Option<Box<dyn ElementInternals>>>,
}

impl RetainedElements {
    pub(crate) fn get_mut(&mut self, element: DynElement) -> &mut dyn ElementInternals {
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
    fn try_get_mut(&mut self, element: DynElement) -> Option<&mut dyn ElementInternals> {
        if element.store_id() != self.id {
            return None;
        }
        self.slots.get_mut(element.key()).and_then(Option::as_deref_mut)
    }

    pub(crate) fn get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> &mut T {
        (self.get_mut(element) as &mut dyn Any)
            .downcast_mut()
            .expect("typed element handle changed type")
    }

    pub(crate) fn try_get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> Option<&mut T> {
        Some(
            (self.try_get_mut(element)? as &mut dyn Any)
                .downcast_mut()
                .expect("typed element handle changed type"),
        )
    }
}

impl Default for Elements {
    fn default() -> Self {
        Self::new()
    }
}

impl Elements {
    pub fn new() -> Self {
        Self {
            elements: RetainedElements {
                id: NEXT_STORE_ID.fetch_add(1, Ordering::Relaxed),
                slots: SlotMap::with_key(),
            },
            states: SlotMap::with_key(),
            by_internal_id: FxHashMap::default(),
            access_tree: RetGuiAccessTree::new(),
            gummy_tree: GummyTree::new(),
            window_manager: WindowManager::new(),
            pending_resources: VecDeque::new(),
            event_queue: VecDeque::with_capacity(10),
            pending_animation_updates: Vec::new(),
            focus: None,
            focus_outline_visible: true,
            #[cfg(feature = "audio")]
            audio_context: None,
            gui_actions: GuiActionQueue::new(),
        }
    }

    /// Inserts an element into this store and creates its layout node.
    pub fn insert_element<T: ElementInternals>(
        &mut self,
        is_scrollable: bool,
        create: impl FnOnce(ElementData) -> T,
    ) -> DynElement {
        let element =
            self.insert_with(|me, access_tree| Box::new(create(ElementData::new(me, is_scrollable, access_tree))));
        self.create_layout_node(element, None);
        element
    }

    /// Runs a local future and applies its result with exclusive access to this
    /// element store on the GUI thread.
    pub fn spawn_local<F, O, C>(&self, future: F, on_complete: C)
    where
        F: Future<Output = O> + 'static,
        O: 'static,
        C: FnOnce(O, &mut Elements) + 'static,
    {
        self.gui_actions.spawn_local(future, on_complete);
    }

    pub(crate) fn run_gui_actions(&mut self) {
        // Collect first so invoking an action never aliases the queue field with
        // the exclusive borrow of the complete store.
        let actions = self.gui_actions.drain();
        for action in actions {
            action(self);
        }
    }

    pub(crate) fn set_gui_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        self.gui_actions.set_waker(waker);
    }

    pub(crate) fn insert_with(
        &mut self,
        create: impl FnOnce(DynElement, RetGuiAccessTree) -> Box<dyn ElementInternals>,
    ) -> DynElement {
        let access_tree = self.access_tree.clone();
        let store_id = self.elements.id;
        let key = self
            .elements
            .slots
            .insert_with_key(|key| Some(create(DynElement::from_key(key, store_id), access_tree)));
        let handle = DynElement::from_key(key, store_id);
        let id = self.get(handle).element_data().internal_id;
        self.by_internal_id.insert(id, handle);
        handle
    }

    pub(crate) fn get(&self, element: DynElement) -> &dyn ElementInternals {
        assert_eq!(
            element.store_id(),
            self.elements.id,
            "element handle belongs to a different store"
        );
        self.elements
            .slots
            .get(element.key())
            .and_then(Option::as_deref)
            .expect("element handle no longer belongs to this store")
    }

    /// Returns a retained element, or `None` when the handle is stale or belongs
    /// to another store.
    pub(crate) fn try_get(&self, element: DynElement) -> Option<&dyn ElementInternals> {
        if element.store_id() != self.elements.id {
            return None;
        }
        self.elements.slots.get(element.key()).and_then(Option::as_deref)
    }

    /// Fast retained-tree lookup for handles already validated when they were
    /// attached to this store.
    pub(crate) fn get_for_draw(&self, element: DynElement) -> &dyn ElementInternals {
        debug_assert_eq!(element.store_id(), self.elements.id);
        self.elements.slots[element.key()]
            .as_deref()
            .expect("retained tree contains a deleted element")
    }

    pub(crate) fn get_mut(&mut self, element: DynElement) -> &mut dyn ElementInternals {
        self.elements.get_mut(element)
    }

    /// Borrows a retained element as its concrete type.
    pub fn get_as<T: ElementInternals>(&self, element: DynElement) -> &T {
        (self.get(element) as &dyn Any)
            .downcast_ref()
            .expect("typed element handle changed type")
    }

    /// Borrows a retained element as its concrete type, returning `None`
    /// for a stale handle.
    pub(crate) fn try_get_as<T: ElementInternals>(&self, element: DynElement) -> Option<&T> {
        Some(
            (self.try_get(element)? as &dyn Any)
                .downcast_ref()
                .expect("typed element handle changed type"),
        )
    }

    /// Mutably borrows a retained element as its concrete type.
    ///
    /// The borrow is tied to this store borrow, just like the framework's own
    /// element-specific setters; no runtime borrow guard is involved.
    pub fn get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> &mut T {
        self.elements.get_as_mut(element)
    }

    /// Mutably borrows a retained element as its concrete type, returning
    /// `None` for a stale handle.
    pub(crate) fn try_get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> Option<&mut T> {
        self.elements.try_get_as_mut(element)
    }

    pub(crate) fn contains(&self, element: DynElement) -> bool {
        element.store_id() == self.elements.id && self.elements.slots.get(element.key()).is_some_and(Option::is_some)
    }

    pub(crate) fn delete_all_children(&mut self, parent: DynElement) {
        let roots = std::mem::take(&mut self.get_mut(parent).element_data_mut().children);
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

        if let Some(focused) = self.focus
            && seen.contains(&focused)
        {
            self.dispatch_mut(focused, |element, elements| element.unfocus(elements));
        }

        if let Some(window) = self.get(parent).element_data().window {
            let capture = &mut self
                .get_as_mut::<crate::elements::WindowElement>(window)
                .pointer_capture;
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
            self.gummy_tree.set_children(parent, &[]);
        }
        for root in layout_roots {
            self.gummy_tree.remove_subtree(root);
        }

        let parent_data = self.get(parent).element_data();
        if let Some(parent_key) = parent_data.access_key {
            parent_data.access_tree.set_children(parent_key, &[]);
        }

        for handle in subtree.into_iter().rev() {
            if let Some(element) = self.elements.slots.remove(handle.key()).flatten() {
                self.by_internal_id.remove(&element.element_data().internal_id);
            }
        }

        self.get(parent).request_window_redraw();
    }

    pub(crate) fn by_internal_id(&self, id: u64) -> Option<DynElement> {
        self.by_internal_id
            .get(&id)
            .copied()
            .filter(|handle| self.contains(*handle))
    }

    pub(crate) fn disjoint_borrow_layout_and_elements(&mut self) -> (&mut GummyTree, &mut RetainedElements) {
        (&mut self.gummy_tree, &mut self.elements)
    }

    pub(crate) fn create_layout_node(&mut self, element: DynElement, context: Option<LayoutContext>) {
        let (tree, elements) = self.disjoint_borrow_layout_and_elements();
        elements
            .get_mut(element)
            .element_data_mut()
            .create_layout_node(tree, context);
    }

    pub(crate) fn with_window_manager<R>(
        &mut self,
        callback: impl FnOnce(&mut WindowManager, &mut Elements) -> R,
    ) -> R {
        let mut manager = std::mem::replace(&mut self.window_manager, WindowManager::new());
        let result = callback(&mut manager, self);
        self.window_manager = manager;
        result
    }

    pub(crate) fn queue_event(&mut self, event: EventKind) {
        self.event_queue.push_back(event);
    }

    pub(crate) fn dequeue_event(&mut self) -> Option<EventKind> {
        self.event_queue.pop_front()
    }

    /// Schedules an element to recompute when it next needs an animation update.
    pub fn schedule_animation_update(&mut self, element: DynElement) {
        self.queue_animation_update(element, false);
    }

    pub(crate) fn restart_animation_update(&mut self, element: DynElement) {
        self.queue_animation_update(element, true);
    }

    fn queue_animation_update(&mut self, element: DynElement, reset_clock: bool) {
        if let Some((_, pending_reset)) = self
            .pending_animation_updates
            .iter_mut()
            .find(|(pending, _)| *pending == element)
        {
            *pending_reset |= reset_clock;
        } else {
            self.pending_animation_updates.push((element, reset_clock));
        }
    }

    pub(crate) fn take_pending_animation_updates(&mut self) -> Vec<(DynElement, bool)> {
        std::mem::take(&mut self.pending_animation_updates)
    }

    #[cfg(feature = "audio")]
    pub(crate) fn with_audio_context<R>(&mut self, callback: impl FnOnce(&mut AudioContext, &mut Elements) -> R) -> R {
        let mut context = self.audio_context.take().unwrap_or_else(AudioContext::new);
        let result = callback(&mut context, self);
        self.audio_context = Some(context);
        result
    }

    /// Mutates one element while retaining exclusive access to the rest of the
    /// store. This is used by tree algorithms that recurse through handles.
    pub(crate) fn dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Elements) -> R,
    ) -> R {
        assert_eq!(
            handle.store_id(),
            self.elements.id,
            "element handle belongs to a different store"
        );
        let mut element = self.elements.slots[handle.key()]
            .take()
            .expect("element is already being visited");
        let result = callback(element.as_mut(), self);
        self.elements.slots[handle.key()] = Some(element);
        result
    }

    /// Mutates a retained element and returns `None` when its handle is stale or
    /// belongs to another store.
    pub(crate) fn try_dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Elements) -> R,
    ) -> Option<R> {
        if handle.store_id() != self.elements.id {
            return None;
        }
        let mut element = self.elements.slots.get_mut(handle.key())?.take()?;
        let result = callback(element.as_mut(), self);
        if let Some(slot) = self.elements.slots.get_mut(handle.key()) {
            *slot = Some(element);
        }
        Some(result)
    }

    /// Stores application state and returns a typed handle to it.
    pub fn insert_state<T: 'static>(&mut self, value: T) -> State<T> {
        State {
            key: self.states.insert(Box::new(value)),
            store_id: self.elements.id,
            marker: PhantomData,
        }
    }

    pub fn state<T: 'static>(&self, state: State<T>) -> &T {
        assert_eq!(
            state.store_id, self.elements.id,
            "state handle belongs to a different store"
        );
        self.states[state.key]
            .downcast_ref()
            .expect("state handle was used with the wrong store")
    }

    pub fn state_mut<T: 'static>(&mut self, state: State<T>) -> &mut T {
        assert_eq!(
            state.store_id, self.elements.id,
            "state handle belongs to a different store"
        );
        self.states[state.key]
            .downcast_mut()
            .expect("state handle was used with the wrong store")
    }
}

/// A copyable, type-safe handle to application state stored in [`Elements`].
///
/// Use [`State::update`] when a mutation should produce a value for a later UI
/// update. The callback only borrows `T`; after it returns, `Elements` is
/// available again for element mutation.
///
/// ```
/// use retgui::elements::Elements;
///
/// let mut elements = Elements::new();
/// let count = elements.insert_state(0_i64);
/// let next = count.update(&mut elements, |count| {
///     *count += 1;
///     *count
/// });
///
/// assert_eq!(next, 1);
/// assert_eq!(*count.read(&elements), 1);
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
    /// Borrows this state value from its arena.
    pub fn read(self, elements: &Elements) -> &T {
        elements.state(self)
    }

    /// Mutably borrows this state value from its arena.
    pub fn write(self, elements: &mut Elements) -> &mut T {
        elements.state_mut(self)
    }

    /// Applies a scoped state mutation.
    ///
    /// The callback intentionally receives only the state value. Any UI work
    /// happens after it returns, when the exclusive state borrow has ended.
    pub fn update<R>(self, elements: &mut Elements, callback: impl FnOnce(&mut T) -> R) -> R {
        callback(elements.state_mut(self))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod async_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::elements::{Container, Element, Elements, Text};

    #[test]
    fn local_completion_updates_the_store_and_wakes_the_driver() {
        let runtime = retgui_runtime::RetGuiRuntime::new();
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "waiting");
        let wakes = Arc::new(AtomicUsize::new(0));
        let wakes_for_callback = wakes.clone();
        elements.set_gui_waker(move || {
            wakes_for_callback.fetch_add(1, Ordering::Relaxed);
        });

        elements.spawn_local(async { "ready" }, move |value, elements| {
            text.set_text(elements, value);
        });

        runtime.handle().update_local_set();
        assert_eq!(text.text(&elements), "waiting");
        assert_eq!(wakes.load(Ordering::Relaxed), 1);

        elements.run_gui_actions();
        assert_eq!(text.text(&elements), "ready");
        elements.run_gui_actions();
        assert_eq!(text.text(&elements), "ready");
    }

    #[test]
    fn local_completion_ignores_a_deleted_target() {
        let runtime = retgui_runtime::RetGuiRuntime::new();
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "waiting");
        let parent = Container::new(&mut elements);
        parent.push(&mut elements, text);

        elements.spawn_local(async { "too late" }, move |value, elements| {
            text.set_text(elements, value);
        });
        parent.delete_all_children(&mut elements);

        runtime.handle().update_local_set();
        elements.run_gui_actions();

        assert!(!elements.contains(text.inner));
    }
}

#[cfg(test)]
mod deletion_tests {
    use crate::elements::{Container, Element, Elements, Text};

    #[test]
    fn deleting_children_reclaims_the_complete_retained_subtree() {
        let mut elements = Elements::new();
        let grandchild = Text::new(&mut elements, "grandchild");
        let child = Container::new(&mut elements);
        child.push(&mut elements, grandchild);
        let parent = Container::new(&mut elements);
        parent.push(&mut elements, child);
        let child_access = elements.get(child.inner).element_data().access_key.unwrap();
        let access_tree = elements.get(child.inner).element_data().access_tree.clone();

        assert_eq!(elements.elements.slots.len(), 3);
        parent.delete_all_children(&mut elements);

        assert!(parent.children(&elements).is_empty());
        assert!(!elements.contains(child.inner));
        assert!(!elements.contains(grandchild.inner));
        assert!(!access_tree.contains_node(child_access));
        assert_eq!(elements.elements.slots.len(), 1);
    }

    #[test]
    fn repeated_child_replacement_does_not_grow_the_arena() {
        let mut elements = Elements::new();
        let parent = Container::new(&mut elements);

        for _ in 0..10 {
            for _ in 0..100 {
                let child = Text::new(&mut elements, "row");
                parent.push(&mut elements, child);
            }
            assert_eq!(elements.elements.slots.len(), 101);
            parent.delete_all_children(&mut elements);
            assert_eq!(elements.elements.slots.len(), 1);
        }
    }

    #[test]
    fn operations_on_deleted_handles_are_inert() {
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "before");
        let child = Container::new(&mut elements);
        child.push(&mut elements, text);
        let parent = Container::new(&mut elements);
        parent.push(&mut elements, child);

        parent.delete_all_children(&mut elements);

        text.set_text(&mut elements, "after");
        text.set_selectable(&mut elements, false);
        text.set_font_size(&mut elements, 24.0);
        text.request_redraw(&elements);

        assert_eq!(text.text(&elements), "");
        assert!(!text.is_selectable(&elements));
        assert!(text.children(&elements).is_empty());
        assert!(matches!(
            text.parent(&elements),
            Err(crate::RetGuiError::ElementNotFound)
        ));
        assert_eq!(elements.elements.slots.len(), 1);
    }
}
