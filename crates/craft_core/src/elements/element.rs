use crate::components::component::{ComponentOrElement, ComponentSpecification};
use crate::components::Event;
use crate::elements::element_data::ElementData;
use crate::elements::element_states::ElementState;
use crate::events::CraftMessage;
use crate::geometry::borders::ComputedBorderSpec;
use crate::geometry::{ElementBox, Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::mem;
use std::sync::Arc;
use taffy::{NodeId, Overflow, TaffyTree};
use winit::event::MouseButton;
use winit::window::Window;
use crate::elements::scroll_state::ScrollState;

#[derive(Clone)]
pub struct ElementBoxed {
    pub internal: Box<dyn Element>,
}

pub trait Element: Any + StandardElementClone + Send + Sync {
    fn element_data(&self) -> &ElementData;
    fn element_data_mut(&mut self) -> &mut ElementData;

    fn children(&self) -> Vec<&dyn Element> {
        self.element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBoxed> {
        &mut self.element_data_mut().children
    }

    fn style(&self) -> &Style {
        &self.element_data().style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.element_data_mut().style
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = self.element_data();
        let rect = element_data.layout_item.computed_box_transformed.border_rectangle();

        if let Some(clip) = element_data.layout_item.clip_bounds {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            rect.contains(&point)
        }
    }

    fn get_id(&self) -> &Option<String> {
        &self.element_data().id
    }

    fn component_id(&self) -> u64 {
        self.element_data().component_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.element_data_mut().component_id = id;
    }

    fn name(&self) -> &'static str;

    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    );

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId>;

    /// Finalizes the layout of the element.
    ///
    /// The majority of the layout computation is done in the `compute_layout` method.
    /// Store the computed values in the `element_data` struct.
    #[allow(clippy::too_many_arguments)]
    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    );

    fn as_any(&self) -> &dyn Any;

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
    ) {
        self.on_style_event(message, element_state, should_style, event);
    }

    fn on_style_event(
        &self,
        message: &CraftMessage,
        _element_state: &mut ElementStateStore,
        should_style: bool,
        _event: &mut Event,
    ) {
        if should_style {
            let state = _element_state.storage.get_mut(&self.element_data().component_id).unwrap();

            match message {
                CraftMessage::PointerMovedEvent(..) => {
                    state.base.hovered = true;
                }
                CraftMessage::PointerButtonEvent(pointer_button) => {
                    if pointer_button.button.mouse_button() == MouseButton::Left
                        && pointer_button.state == winit::event::ElementState::Pressed
                    {
                        state.base.active = true;
                    }
                }
                _ => {}
            }
        }
    }

    fn resolve_clip(
        &mut self,
        clip_bounds: Option<Rectangle>,
    ) {
        self.element_data_mut().layout_item.resolve_clip(clip_bounds);
    }

    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: glam::Mat4,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
        let position = self.element_data().style.position();
        self.element_data_mut().layout_item.resolve_box(relative_position, scroll_transform, result, layout_order, position);
    }

    fn draw_children(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        for child in self.element_data_mut().children.iter_mut() {
            let taffy_child_node_id = child.internal.taffy_node_id();
            // Skip non-visual elements.
            if taffy_child_node_id.is_none() {
                continue;
            }
            child.internal.draw(
                renderer,
                text_context,
                taffy_tree,
                taffy_child_node_id.unwrap(),
                element_state,
                pointer,
                window.clone(),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut RenderList, element_state: &mut ElementStateStore) {
        let base_state = self.get_base_state(element_state);
        let current_style = base_state.base.current_style(self.element_data());

        self.element_data().layout_item.draw_borders(renderer, current_style);
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.current_style().overflow()[1] == Overflow::Scroll
    }

    fn maybe_start_layer(&self, renderer: &mut RenderList) {
        let element_data = self.element_data();
        let padding_rectangle = element_data.layout_item.computed_box_transformed.padding_rectangle();

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut RenderList) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn finalize_borders(&mut self, element_state: &ElementStateStore) {
        let base_state = self.get_base_state(element_state);
        let (has_border, border_radius, border_color) = {
            let current_style = base_state.base.current_style(&self.element_data());
            (current_style.has_border(), current_style.border_radius(), current_style.border_color())
        };

        self.element_data_mut().layout_item.finalize_borders(has_border, border_radius, border_color);
    }

    fn draw_scrollbar(&mut self, renderer: &mut RenderList) {
        let scrollbar_color = self.element_data().current_style().scrollbar_color();

        // track
        renderer.draw_rect(self.element_data_mut().layout_item.computed_scroll_track, scrollbar_color.track_color);

        // thumb
        renderer.draw_rect(self.element_data_mut().layout_item.computed_scroll_thumb, scrollbar_color.thumb_color);
    }

    fn finalize_scrollbar(&mut self, scroll_state: &mut ScrollState) {
        let element_data = self.element_data_mut();
        scroll_state.finalize_layout(element_data);
    }

    /// Called when the element is assigned a unique component id.
    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(()),
        }
    }

    #[allow(dead_code)]
    fn finalize_state(&mut self, element_state: &mut ElementStateStore, pointer: Option<Point>) {
        let element_data = self.element_data_mut();
        let element_state = element_state.storage.get_mut(&element_data.component_id).unwrap();
        element_state.base.current_state = ElementState::Normal;

        let border_rectangle = element_data.layout_item.computed_box_transformed.border_rectangle();

        if let Some(pointer) = pointer {
            if border_rectangle.contains(&pointer) {
                element_state.base.current_state = ElementState::Hovered;
            }
        }
    }

    fn get_base_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a ElementStateStoreItem {
        element_state.storage.get(&self.element_data().component_id).unwrap()
    }

    #[allow(dead_code)]
    fn get_base_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut ElementStateStoreItem {
        element_state.storage.get_mut(&self.element_data().component_id).unwrap()
    }

    /// Called on sequential renders to update any state that the element may have.
    fn update_state(&mut self, _element_state: &mut ElementStateStore, _reload_fonts: bool, _scaling_factor: f64) {}

    fn default_style(&self) -> Style {
        Style::default()
    }

    fn merge_default_style(&mut self) {
        self.element_data_mut().style = Style::merge(&self.default_style(), &self.element_data().style);
    }

    
    // Easy ways to access common items from layout item:
    fn taffy_node_id(&self) -> Option<NodeId> {
        self.element_data().layout_item.taffy_node_id
    }
    
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout_item.computed_box
    }
    
    fn computed_box_transformed(&self) -> ElementBox {
        self.element_data().layout_item.computed_box_transformed
    }

    fn computed_border(&self) -> &ComputedBorderSpec {
        &self.element_data().layout_item.computed_border
    }
}

impl<T: Element> From<T> for ElementBoxed {
    fn from(element: T) -> Self {
        ElementBoxed {
            internal: Box::new(element),
        }
    }
}

impl<T: Element> From<T> for ComponentOrElement {
    fn from(element: T) -> Self {
        ComponentOrElement::Element(element.into())
    }
}

impl From<ElementBoxed> for ComponentOrElement {
    fn from(element: ElementBoxed) -> Self {
        ComponentOrElement::Element(element)
    }
}

impl From<ElementBoxed> for ComponentSpecification {
    fn from(mut element: ElementBoxed) -> Self {
        let data = element.internal.element_data_mut();

        let key = mem::take(&mut data.key);
        let children = mem::take(&mut data.child_specs);
        let props = mem::take(&mut data.props);

        ComponentSpecification {
            component: ComponentOrElement::Element(element),
            key,
            props,
            children,
        }
    }
}

impl<T> From<T> for ComponentSpecification
where
    T: Element,
{
    fn from(mut element: T) -> Self {
        let data = element.element_data_mut();

        let key = mem::take(&mut data.key);
        let children_specs = mem::take(&mut data.child_specs);
        let props = mem::take(&mut data.props);

        ComponentSpecification {
            component: ComponentOrElement::Element(element.into()),
            key,
            props,
            children: children_specs,
        }
    }
}

pub trait StandardElementClone {
    fn clone_box(&self) -> Box<dyn Element>;
}

impl<T> StandardElementClone for T
where
    T: Element + Clone,
{
    fn clone_box(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Element> {
    fn clone(&self) -> Box<dyn Element> {
        self.clone_box()
    }
}

#[macro_export]
macro_rules! generate_component_methods_no_children {
    () => {
        #[allow(dead_code)]
        pub fn component(self) -> ComponentSpecification {
            ComponentSpecification::new(self.into())
        }

        #[allow(dead_code)]
        pub fn key(mut self, key: &str) -> Self {
            self.element_data.key = Some(key.to_string());
            self
        }

        #[allow(dead_code)]
        pub fn props(mut self, props: Props) -> Self {
            self.element_data.props = Some(props);
            self
        }

        #[allow(dead_code)]
        pub fn id(mut self, id: &str) -> Self {
            self.element_data.id = Some(id.to_string());
            self
        }

        #[allow(dead_code)]
        pub fn normal(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Normal;
            self
        }

        #[allow(dead_code)]
        pub fn hovered(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Hovered;
            self
        }

        #[allow(dead_code)]
        pub fn pressed(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Pressed;
            self
        }

        #[allow(dead_code)]
        pub fn disabled(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Disabled;
            self
        }

        #[allow(dead_code)]
        pub fn focused(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Focused;
            self
        }

        #[allow(dead_code)]
        /// Sets the on_pointer_button handler for the element.
        pub fn on_pointer_button<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::PointerButton)
                + Send
                + Sync
                + 'static,
        {
            use $crate::components::Event;
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::PointerButton;

            let callback: EventHandlerWithRef<PointerButton> = Arc::new(
                move |state_any: &mut dyn Any,
                      global_any: &mut dyn Any,
                      event: &mut Event,
                      pointer_button: &PointerButton| {
                    let state = state_any.downcast_mut::<State>().unwrap();
                    let global = global_any.downcast_mut::<GlobalState>().unwrap();
                    handler(state, global, event, pointer_button);
                },
            );
            self.element_data_mut().on_pointer_button = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_initialized handler for the element.
        pub fn on_initialized<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event) + Send + Sync + 'static,
        {
            use $crate::components::Event;
            use $crate::elements::element_data::EventHandler;

            let callback: EventHandler =
                Arc::new(move |state_any: &mut dyn Any, global_any: &mut dyn Any, event: &mut Event| {
                    let state = state_any.downcast_mut::<State>().unwrap();
                    let global = global_any.downcast_mut::<GlobalState>().unwrap();
                    handler(state, global, event);
                });
            self.element_data_mut().on_initialized = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_keyboard_input handler for the element.
        pub fn on_keyboard_input<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::KeyboardInput)
                + Send
                + Sync
                + 'static,
        {
            use $crate::components::Event;
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::KeyboardInput;

            let callback: EventHandlerWithRef<KeyboardInput> = Arc::new(
                move |state_any: &mut dyn Any,
                      global_any: &mut dyn Any,
                      event: &mut Event,
                      keyboard_input: &KeyboardInput| {
                    let state = state_any.downcast_mut::<State>().unwrap();
                    let global = global_any.downcast_mut::<GlobalState>().unwrap();
                    handler(state, global, event, keyboard_input);
                },
            );
            self.element_data_mut().on_keyboard_input = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_pointer_move handler for the element.
        pub fn on_pointer_move<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::PointerMoved)
                + Send
                + Sync
                + 'static,
        {
            use $crate::components::Event;
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::PointerMoved;

            let callback: EventHandlerWithRef<PointerMoved> = Arc::new(
                move |state_any: &mut dyn Any,
                      global_any: &mut dyn Any,
                      event: &mut Event,
                      pointer_moved: &PointerMoved| {
                    let state = state_any.downcast_mut::<State>().unwrap();
                    let global = global_any.downcast_mut::<GlobalState>().unwrap();
                    handler(state, global, event, pointer_moved);
                },
            );
            self.element_data_mut().on_pointer_move = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_mouse_wheel handler for the element.
        pub fn on_mouse_wheel<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::MouseWheel)
                + Send
                + Sync
                + 'static,
        {
            use $crate::components::Event;
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::MouseWheel;

            let callback: EventHandlerWithRef<MouseWheel> = Arc::new(
                move |state_any: &mut dyn Any,
                      global_any: &mut dyn Any,
                      event: &mut Event,
                      mouse_wheel: &MouseWheel| {
                    let state = state_any.downcast_mut::<State>().unwrap();
                    let global = global_any.downcast_mut::<GlobalState>().unwrap();
                    handler(state, global, event, mouse_wheel);
                },
            );
            self.element_data_mut().on_mouse_wheel = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_modifiers_changed handler for the element.
        pub fn on_modifiers_changed<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::Modifiers)
                + Send
                + Sync
                + 'static,
        {
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::Modifiers;

            let callback: EventHandlerWithRef<Modifiers> = Arc::new(move |state_any, global_any, event, modifiers| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, modifiers);
            });
            self.element_data_mut().on_modifiers_changed = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_ime handler for the element.
        pub fn on_ime<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &$crate::events::Ime)
                + Send
                + Sync
                + 'static,
        {
            use $crate::elements::element_data::EventHandlerWithRef;
            use $crate::events::Ime;

            let callback: EventHandlerWithRef<Ime> = Arc::new(move |state_any, global_any, event, ime| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, ime);
            });
            self.element_data_mut().on_ime = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_text_input_changed handler for the element.
        pub fn on_text_input_changed<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, &str) + Send + Sync + 'static,
        {
            use $crate::elements::element_data::EventHandlerWithRef;

            let callback: EventHandlerWithRef<str> = Arc::new(move |state_any, global_any, event, text| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, text);
            });
            self.element_data_mut().on_text_input_changed = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_dropdown_toggled handler for the element.
        pub fn on_dropdown_toggled<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, bool) + Send + Sync + 'static,
        {
            use $crate::elements::element_data::EventHandlerCopy;

            let callback: EventHandlerCopy<bool> = Arc::new(move |state_any, global_any, event, flag| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, flag);
            });
            self.element_data_mut().on_dropdown_toggled = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_dropdown_item_selected handler for the element.
        pub fn on_dropdown_item_selected<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, usize) + Send + Sync + 'static,
        {
            use $crate::elements::element_data::EventHandlerCopy;

            let callback: EventHandlerCopy<usize> = Arc::new(move |state_any, global_any, event, index| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, index);
            });
            self.element_data_mut().on_dropdown_item_selected = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_switch_toggled handler for the element.
        pub fn on_switch_toggled<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, bool) + Send + Sync + 'static,
        {
            use $crate::elements::element_data::EventHandlerCopy;

            let callback: EventHandlerCopy<bool> = Arc::new(move |state_any, global_any, event, flag| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, flag);
            });
            self.element_data_mut().on_switch_toggled = Some(callback);
            self
        }

        #[allow(dead_code)]
        /// Sets the on_slider_value_changed handler for the element.
        pub fn on_slider_value_changed<State, GlobalState, Handler>(mut self, handler: Handler) -> Self
        where
            State: Any + Send + Sync + 'static,
            GlobalState: Any + Send + Sync + Default + 'static,
            Handler: Fn(&mut State, &mut GlobalState, &mut $crate::components::Event, f64) + Send + Sync + 'static,
        {
            use $crate::elements::element_data::EventHandlerCopy;

            let callback: EventHandlerCopy<f64> = Arc::new(move |state_any, global_any, event, value| {
                let state = state_any.downcast_mut::<State>().unwrap();
                let global = global_any.downcast_mut::<GlobalState>().unwrap();
                handler(state, global, event, value);
            });
            self.element_data_mut().on_slider_value_changed = Some(callback);
            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods_private_push {
    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods {
    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        pub fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        pub fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        pub fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        pub fn push_in_place(&mut self, component_specification: ComponentSpecification) {
            self.element_data.child_specs.push(component_specification);
        }
    };
}

pub(crate) fn resolve_clip_for_scrollable(element: &mut dyn Element, clip_bounds: Option<Rectangle>) {
    let element_data = element.element_data_mut();
    if element_data.is_scrollable() {
        let scroll_clip_bounds = element_data.layout_item.computed_box_transformed.padding_rectangle();
        if let Some(clip_bounds) = clip_bounds {
            element_data.layout_item.clip_bounds = scroll_clip_bounds.intersection(&clip_bounds);
        } else {
            element_data.layout_item.clip_bounds = Some(scroll_clip_bounds);
        }
    } else {
        element_data.layout_item.clip_bounds = clip_bounds;
    }
}