use crate::accessibility::access_handler::CraftAccessHandler;
use crate::accessibility::activation_handler::CraftActivationHandler;
use crate::accessibility::deactivation_handler::CraftDeactivationHandler;
use crate::components::{ComponentSpecification, Event};
use crate::craft_runtime::CraftRuntimeHandle;
use crate::devtools::dev_tools_component::dev_tools_view;
use crate::elements::{Container, Element};
use crate::events::event_dispatch::dispatch_event;
use crate::events::internal::{InternalMessage, InternalUserMessage};
use crate::events::resource_event::ResourceEvent;
use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::geometry::{Rectangle, Size};
use crate::layout::layout_context::{measure_content, LayoutContext};
use crate::reactive::element_id::reset_unique_element_id;
use crate::reactive::element_state_store::ElementStateStore;
use crate::reactive::reactive_tree::ReactiveTree;
use crate::reactive::tree::diff_trees;
use crate::renderer::RenderList;
use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::style::{Display, Unit, Wrap};
use crate::text::text_context::TextContext;
use crate::view_introspection::scan_view_for_resources;
use crate::{GlobalState, RendererBox, WindowContext};
use accesskit::{Role, TreeUpdate};
use accesskit_winit::Adapter;
use cfg_if::cfg_if;
use craft_logging::{info, span, Level};
use kurbo::Point;
use peniko::Color;
use std::collections::HashMap;
use std::sync::Arc;
use taffy::{AvailableSpace, NodeId, TaffyTree};
use tokio::sync::mpsc::Sender;
use ui_events::keyboard::{KeyState, KeyboardEvent};
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
use winit::dpi::PhysicalSize;
use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub(crate) struct App {
    /// The user's view specification. This is lazily evaluated and will be called each time the view is redrawn.
    pub(crate) app: ComponentSpecification,
    /// The global state is used to store global data that can be accessed from anywhere in the user's application.
    pub(crate) global_state: GlobalState,
    /// A winit window. This is only valid between resume and pause.
    pub(crate) window: Option<Arc<Window>>,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    pub(crate) text_context: Option<TextContext>,
    /// The renderer is used to draw the view. It is only valid between resume and pause.
    pub(crate) renderer: Option<RendererBox>,
    pub(crate) mouse_position: Option<Point>,
    pub(crate) reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,
    /// Resources that have already been collected.
    /// We use this in view_introspection, so that we don't request the download
    /// of a resource too many times.
    pub(crate) resources_collected: HashMap<ResourceIdentifier, bool>,
    // The user's reactive tree.
    pub(crate) user_tree: ReactiveTree,
    /// Provides a way for the user to get and set common window properties during view and update.
    pub(crate) window_context: WindowContext,

    #[cfg(feature = "dev_tools")]
    pub(crate) is_dev_tools_open: bool,

    /// The dev tools tree is used to display the reactive tree in the dev tools.
    #[cfg(feature = "dev_tools")]
    pub(crate) dev_tree: ReactiveTree,
    pub(crate) app_sender: Sender<InternalMessage>,
    pub(crate) accesskit_adapter: Option<Adapter>,
    pub(crate) runtime: CraftRuntimeHandle,
}

impl App {
    pub(crate) fn on_close_requested(&mut self) {
        info!("Craft application is closing.");
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        self.window_context.scale_factor = scale_factor;
        self.on_resize(self.window.as_ref().unwrap().inner_size());
    }

    pub(crate) fn on_process_user_events(&mut self, is_dev_tree: bool) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };

        if reactive_tree.update_queue.is_empty() {
            return;
        }

        for event in reactive_tree.update_queue.drain(..) {
            let app_sender_copy = self.app_sender.clone();
            let f = async move {
                let update_result = event.update_result.unwrap();
                let res = update_result.await;
                app_sender_copy
                    .send(InternalMessage::GotUserMessage(InternalUserMessage {
                        update_fn: event.update_function,
                        source_component_id: event.source_component,
                        message: res,
                        props: event.props,
                    }))
                    .await
                    .expect("Failed to send user message");
            };
            self.runtime.spawn(f);
        }
    }

    pub(crate) fn on_resume(&mut self, window: Arc<Window>, renderer: RendererBox, event_loop: &ActiveEventLoop) {
        window.set_ime_allowed(true);

        if self.user_tree.element_tree.is_none() {
            reset_unique_element_id();
        }

        self.setup_text_context();
        self.renderer = Some(renderer);

        self.window = Some(window.clone());

        #[cfg(not(target_arch = "wasm32"))]
        let action_handler = CraftAccessHandler {
            runtime_handle: self.runtime.clone(),
            app_sender: self.app_sender.clone(),
        };
        let deactivation_handler = CraftDeactivationHandler::new();

        let scale_factor = window.scale_factor();

        self.window = Some(window.clone());
        self.window_context.scale_factor = scale_factor;
        self.on_resize(window.inner_size());
        let tree_update = self.on_request_redraw();

        let craft_activation_handler = CraftActivationHandler::new(tree_update);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.accesskit_adapter = Some(Adapter::with_direct_handlers(
                event_loop,
                &window,
                craft_activation_handler,
                action_handler,
                deactivation_handler,
            ));
        }

        window.set_visible(true);
    }

    /// Handles the window resize event.
    pub(crate) fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_context.window_size = new_size;
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);
        }
        // On macOS the window needs to be redrawn manually after resizing
        #[cfg(target_os = "macos")]
        {
            self.window.as_ref().unwrap().request_redraw();
        }
    }

    /// Initialize any data needed to layout/render text.
    fn setup_text_context(&mut self) {
        if self.text_context.is_none() {
            #[cfg(target_arch = "wasm32")]
            let mut text_context = TextContext::new();
            #[cfg(not(target_arch = "wasm32"))]
            let text_context = TextContext::new();

            #[cfg(target_arch = "wasm32")]
            {
                let variable_roboto = include_bytes!("../../../fonts/Roboto-VariableFont_wdth,wght.ttf");
                let roboto_blog = peniko::Blob::new(Arc::new(variable_roboto));
                let fonts = text_context.font_context.collection.register_fonts(roboto_blog, None);

                // Register all the Roboto families under parley::GenericFamily::SystemUi.
                // This will become the fallback font for platforms like WASM.
                text_context
                    .font_context
                    .collection
                    .append_generic_families(parley::GenericFamily::SystemUi, fonts.iter().map(|f| f.0));
            }

            self.text_context = Some(text_context);
        }
    }

    /// Updates the view by applying the latest changes to the reactive tree.
    pub(crate) fn update_view(&mut self) {
        self.setup_text_context();
        let text_context = self.text_context.as_mut().unwrap();

        let old_element_ids = self.user_tree.element_ids.clone();
        let old_component_ids = self.user_tree.component_ids.clone();
        update_reactive_tree(
            self.app.clone(),
            &mut self.user_tree,
            &mut self.global_state,
            &mut self.reload_fonts,
            text_context,
            self.window_context.scale_factor,
            &mut self.window_context,
        );

        // Cleanup unmounted components and elements.
        self.user_tree.user_state.remove_unused_state(&old_component_ids, &self.user_tree.component_ids);
        self.user_tree.element_state.remove_unused_state(&old_element_ids, &self.user_tree.element_ids);
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    pub(crate) fn on_request_redraw(&mut self) -> Option<TreeUpdate> {
        self.window.as_ref()?;

        let window = self.window.as_mut().unwrap().clone();

        let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

        self.setup_text_context();

        self.update_view();

        cfg_if! {
            if #[cfg(feature = "dev_tools")] {
                let mut root_size = surface_size;
            } else {
                let root_size = surface_size;
            }
        }

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().surface_set_clear_color(Color::WHITE);
        }

        #[cfg(feature = "dev_tools")]
        {
            if self.is_dev_tools_open {
                let dev_tools_size = Size::new(350.0, root_size.height);
                root_size.width -= dev_tools_size.width;
            }
        }

        {
            self.layout_tree(
                false,
                root_size,
                Point::new(0.0, 0.0),
                self.window_context.scale_factor,
                self.mouse_position,
            );

            if self.renderer.is_some() {
                self.draw_reactive_tree(false, self.mouse_position, self.window.clone());
            }
        }

        #[cfg(feature = "dev_tools")]
        {
            if self.is_dev_tools_open {
                update_reactive_tree(
                    dev_tools_view(self.user_tree.element_tree.clone().unwrap()),
                    &mut self.dev_tree,
                    &mut self.global_state,
                    &mut self.reload_fonts,
                    self.text_context.as_mut().unwrap(),
                    self.window_context.scale_factor,
                    &mut self.window_context,
                );

                self.layout_tree(
                    true,
                    Size::new(surface_size.width - root_size.width, root_size.height),
                    Point::new(root_size.width as f64, 0.0),
                    self.window_context.scale_factor,
                    self.mouse_position,
                );

                if self.renderer.is_some() {
                    self.draw_reactive_tree(true, self.mouse_position, self.window.clone());
                }
            }
        }

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().submit(self.resource_manager.clone());
        }

        if let Some(window) = &self.window {
            self.window_context.apply_requests(window);
            self.window_context.reset();
        }

        self.on_process_user_events(false);
        #[cfg(feature = "dev_tools")]
        {
            self.on_process_user_events(true);
        }

        self.view_introspection();

        let tree_update = self.compute_accessibility_tree();
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            accesskit_adapter.update_if_active(|| tree_update);
            window.pre_present_notify();
            None
        } else {
            window.pre_present_notify();
            Some(tree_update)
        }
    }

    pub(crate) fn on_pointer_scroll(&mut self, pointer_scroll_update: PointerScrollUpdate) {
        let event = CraftMessage::PointerScroll(pointer_scroll_update);
        let message = Message::CraftMessage(event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);
        self.request_redraw();
    }

    pub(crate) fn on_pointer_button(
        &mut self,
        pointer_event: PointerButtonUpdate,
        is_up: bool,
        dispatch_type: EventDispatchType,
    ) {
        let cursor_position = pointer_event.state.position;

        let event = if is_up {
            CraftMessage::PointerButtonUp(pointer_event)
        } else {
            CraftMessage::PointerButtonDown(pointer_event)
        };
        let message = Message::CraftMessage(event);

        self.window_context.mouse_position = Some(Point::new(cursor_position.x, cursor_position.y));

        if let EventDispatchType::Direct(component) = dispatch_type {
            self.dispatch_event(&message, EventDispatchType::Direct(component), false);
        } else {
            self.dispatch_event(&message, EventDispatchType::Bubbling, true);
        }

        self.request_redraw();
    }

    pub(crate) fn on_pointer_moved(&mut self, mouse_moved: PointerUpdate) {
        self.mouse_position = Some(mouse_moved.current.position);
        self.window_context.mouse_position = Some(mouse_moved.current.position);

        let message = Message::CraftMessage(CraftMessage::PointerMovedEvent(mouse_moved));

        self.dispatch_event(&message, EventDispatchType::Bubbling, true);

        self.request_redraw();
    }

    pub(crate) fn on_ime(&mut self, ime: Ime) {
        let event = CraftMessage::ImeEvent(ime);
        let message = Message::CraftMessage(event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);

        self.request_redraw();
    }

    /// Dispatch messages to the reactive tree.
    fn dispatch_event(&mut self, message: &Message, dispatch_type: EventDispatchType, is_style: bool) {
        dispatch_event(
            message,
            dispatch_type.clone(),
            &mut self.resource_manager,
            self.mouse_position,
            &mut self.user_tree,
            &mut self.global_state,
            &mut self.text_context,
            &mut self.window_context,
            is_style,
        );

        #[cfg(feature = "dev_tools")]
        dispatch_event(
            message,
            dispatch_type,
            &mut self.resource_manager,
            self.mouse_position,
            &mut self.dev_tree,
            &mut self.global_state,
            &mut self.text_context,
            &mut self.window_context,
            is_style,
        );
    }

    pub(crate) fn on_keyboard_input(&mut self, keyboard_input: KeyboardEvent) {
        let keyboard_event = CraftMessage::KeyboardInputEvent(keyboard_input.clone());
        let message = Message::CraftMessage(keyboard_event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);

        #[cfg(feature = "dev_tools")]
        {
            let logical_key = keyboard_input.key;
            let key_state = keyboard_input.state;

            if KeyState::Down == key_state {
                if let ui_events::keyboard::Key::Named(ui_events::keyboard::NamedKey::F12) = logical_key {
                    self.is_dev_tools_open = !self.is_dev_tools_open;
                }
            }
        }

        self.request_redraw();
    }

    /// Processes async messages sent from the user.
    pub(crate) fn on_user_message(&mut self, message: InternalUserMessage) {
        let state = self.user_tree.user_state.storage.get_mut(&message.source_component_id).unwrap().as_mut();

        let mut event = Event::with_window_context(self.window_context.clone());

        (message.update_fn)(
            state,
            &mut self.global_state,
            message.props,
            &mut event,
            &Message::UserMessage(message.message),
        );
        self.window_context = event.window;
    }

    pub(crate) fn on_resource_event(&mut self, resource_event: ResourceEvent) {
        match resource_event {
            ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                if resource_type == ResourceType::Font {
                    if let Some(_text_context) = self.text_context.as_mut() {
                        if resource.data().is_some() {
                            // Todo: Load the font into the text context.
                            self.resource_manager.resources.insert(resource_identifier.clone(), Arc::new(resource));
                        }
                    }

                    self.reload_fonts = true;
                } else if resource_type == ResourceType::Image || resource_type == ResourceType::TinyVg {
                    self.resource_manager.resources.insert(resource_identifier, Arc::new(resource));
                }
            }
            ResourceEvent::UnLoaded(_) => {}
        }
    }

    fn view_introspection(&mut self) {
        scan_view_for_resources(
            self.user_tree.element_tree.as_ref().unwrap().as_ref(),
            self.user_tree.component_tree.as_ref().unwrap(),
            self.resource_manager.clone(),
            &mut self.resources_collected,
        );
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_tree(
        &mut self,
        is_dev_tree: bool,
        viewport_size: Size<f32>,
        origin: Point,
        scale_factor: f64,
        mouse_position: Option<Point>,
    ) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };
        let root_element = reactive_tree.element_tree.as_mut().unwrap();

        let mut root_size = viewport_size;

        // When we lay out the root element it scales up the values by the scale factor, so we need to scale it down here.
        // We do not want to scale the window size.
        {
            root_size.width /= scale_factor as f32;
            root_size.height /= scale_factor as f32;
        }

        style_root_element(root_element, root_size);
        let text_context = self.text_context.as_mut().unwrap();

        {
            let span = span!(Level::INFO, "layout");
            let _enter = span.enter();
            layout(
                &mut reactive_tree.element_state,
                root_size.width,
                root_size.height,
                text_context,
                root_element.as_mut(),
                origin,
                self.resource_manager.clone(),
                scale_factor,
                mouse_position,
            )
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_reactive_tree(&mut self, is_dev_tree: bool, mouse_position: Option<Point>, window: Option<Arc<Window>>) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };
        let root_element = reactive_tree.element_tree.as_mut().unwrap();

        let text_context = self.text_context.as_mut().unwrap();
        {
            let span = span!(Level::INFO, "render");
            let _enter = span.enter();
            let mut render_list = RenderList::new();
            root_element.draw(&mut render_list, text_context, &mut reactive_tree.element_state, mouse_position, window);

            let renderer = self.renderer.as_mut().unwrap();
            renderer.sort_and_cull_render_list(&mut render_list);

            let window = Rectangle {
                x: 0.0,
                y: 0.0,
                width: renderer.surface_width(),
                height: renderer.surface_height(),
            };
            renderer.prepare_render_list(render_list, self.resource_manager.clone(), window);
        }
    }

    fn compute_accessibility_tree(&mut self) -> TreeUpdate {
        let tree = accesskit::Tree {
            root: accesskit::NodeId(0),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(0),
        };

        let state = &mut self.user_tree.element_state;

        self.user_tree.element_tree.as_mut().unwrap().compute_accessibility_tree(&mut tree_update, None, state);
        tree_update.nodes[0].1.set_role(Role::Window);

        tree_update
    }
}

#[allow(clippy::too_many_arguments)]
fn update_reactive_tree(
    component_spec_to_generate_tree: ComponentSpecification,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    should_reload_fonts: &mut bool,
    text_context: &mut TextContext,
    scaling_factor: f64,
    window_context: &mut WindowContext,
) {
    let window_element = Container::new().into();
    let old_component_tree = reactive_tree.component_tree.as_ref();

    let new_tree = {
        let span = span!(Level::INFO, "reactive tree diffing");
        let _enter = span.enter();
        diff_trees(
            component_spec_to_generate_tree.clone(),
            window_element,
            old_component_tree,
            &mut reactive_tree.user_state,
            global_state,
            &mut reactive_tree.element_state,
            *should_reload_fonts,
            text_context,
            scaling_factor,
            window_context,
            &mut reactive_tree.update_queue,
        )
    };

    *should_reload_fonts = false;

    reactive_tree.element_tree = Some(new_tree.element_tree.internal);
    reactive_tree.component_tree = Some(new_tree.component_tree);
    reactive_tree.component_ids = new_tree.component_ids;
    reactive_tree.element_ids = new_tree.element_ids;
    reactive_tree.pointer_captures = new_tree.pointer_captures;
}

fn style_root_element(root: &mut Box<dyn Element>, root_size: Size<f32>) {
    *root.style_mut().width_mut() = Unit::Px(root_size.width);
    *root.style_mut().wrap_mut() = Wrap::Wrap;
    *root.style_mut().display_mut() = Display::Block;

    let is_user_root_height_auto = {
        let root_children = root.children_mut();
        root_children[0].internal.style().height().is_auto()
    };

    *root.style_mut().width_mut() = Unit::Px(root_size.width);
    *root.style_mut().wrap_mut() = Wrap::Wrap;
    *root.style_mut().display_mut() = Display::Block;

    if is_user_root_height_auto {
        *root.style_mut().height_mut() = Unit::Auto;
    } else {
        *root.style_mut().height_mut() = Unit::Px(root_size.height);
        *root.style_mut().height_mut() = Unit::Px(root_size.height);
    }
}

#[allow(clippy::too_many_arguments)]
fn layout(
    element_state: &mut ElementStateStore,
    window_width: f32,
    window_height: f32,
    text_context: &mut TextContext,
    root_element: &mut dyn Element,
    origin: Point,
    resource_manager: Arc<ResourceManager>,
    scale_factor: f64,
    pointer: Option<Point>,
) -> (TaffyTree<LayoutContext>, NodeId) {
    let mut taffy_tree: TaffyTree<LayoutContext> = TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, element_state, scale_factor).unwrap();

    let available_space: taffy::Size<AvailableSpace> = taffy::Size {
        width: AvailableSpace::Definite(window_width),
        height: AvailableSpace::Definite(window_height),
    };

    taffy_tree
        .compute_layout_with_measure(
            root_node,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, style| {
                measure_content(
                    element_state,
                    known_dimensions,
                    available_space,
                    node_context,
                    text_context,
                    resource_manager.clone(),
                    style,
                )
            },
        )
        .unwrap();

    let transform = glam::Mat4::IDENTITY;

    let mut layout_order: u32 = 0;
    root_element.finalize_layout(
        &mut taffy_tree,
        root_node,
        origin,
        &mut layout_order,
        transform,
        element_state,
        pointer,
        text_context,
        None,
    );

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}
