use crate::events::EventDispatcher;
use crate::events::internal::InternalMessage;
use crate::events::{CraftMessage};
use crate::layout::layout_context::measure_content;
use crate::style::{Display, Unit, Wrap};
use crate::text::text_context::TextContext;
use crate::{RendererBox, WindowContext};
use craft_logging::{info, span, Level};
use craft_primitives::geometry::Rectangle;
use craft_resource_manager::{ResourceIdentifier, ResourceManager};
use craft_runtime::CraftRuntimeHandle;
use kurbo::{Affine, Point};
use peniko::Color;
use std::cell::{Cell};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use {
    crate::accessibility::access_handler::CraftAccessHandler,
    crate::accessibility::activation_handler::CraftActivationHandler,
    crate::accessibility::deactivation_handler::CraftDeactivationHandler,
};
#[cfg(feature = "accesskit")]
use {
    accesskit::{Role, TreeUpdate},
    accesskit_winit::Adapter,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;

use understory_box_tree::Tree as SpatialTree;

use crate::animations::animation::AnimationFlags;
use crate::document::DocumentManager;
use crate::elements::Element;
use crate::layout::layout_context::LayoutContext;
use craft_renderer::RenderList;
use craft_resource_manager::resource_event::ResourceEvent;
use craft_resource_manager::resource_type::ResourceType;
use craft_runtime::Sender;
use std::time::Duration;
use taffy::{AvailableSpace, NodeId, TaffyTree};
use ui_events::keyboard::{KeyboardEvent, Modifiers, NamedKey};
use ui_events::pointer::{PointerButtonEvent, PointerScrollEvent, PointerUpdate};
use ui_events::ScrollDelta;
use ui_events::ScrollDelta::PixelDelta;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

thread_local! {
    /// The most recently recorded window id. This is set every time a windows event occurs.
    pub static CURRENT_WINDOW_ID : Cell<Option<WindowId>> = const { Cell::new(None) };
    /// Records document-level state (focus, pointer captures, etc.) for internal use.
    pub static DOCUMENTS: RefCell<DocumentManager> = RefCell::new(DocumentManager::new());
    pub(crate) static TAFFY_TREE: RefCell<TaffyTree<LayoutContext>> = RefCell::new(TaffyTree::new());
    pub(crate) static SPATIAL_TREE: RefCell<SpatialTree<understory_index::RTreeF64<()>>> = RefCell::new(SpatialTree::<understory_index::RTreeF64<()>>::default());
    pub(crate) static SPATIAL_TREE_MAP: RefCell<HashMap<understory_box_tree::NodeId, Weak<RefCell<dyn Element>>>> = RefCell::new(HashMap::new());
    pub(crate) static PENDING_RESOURCES: RefCell<VecDeque<(ResourceIdentifier, ResourceType)>> = RefCell::new(VecDeque::new());
    pub(crate) static IN_PROGRESS_RESOURCES: RefCell<VecDeque<(ResourceIdentifier, ResourceType)>> = RefCell::new(VecDeque::new());
    pub(crate) static FOCUS: RefCell<Option<Weak<RefCell<dyn Element>>>> = RefCell::new(None);
}

pub struct App {
    pub(crate) event_dispatcher: EventDispatcher,
    pub(crate) root: Rc<RefCell<dyn crate::elements::Element>>,
    /// A winit window. This is only valid between resume and pause.
    pub window: Option<Arc<Window>>,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    pub(crate) text_context: Option<TextContext>,
    /// The renderer is used to draw the view. It is only valid between resume and pause.
    pub renderer: Option<RendererBox>,
    pub(crate) reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,
    // The user's reactive tree.
    /// Provides a way for the user to get and set common window properties during view and update.
    pub window_context: WindowContext,

    pub(crate) app_sender: Sender<InternalMessage>,
    #[cfg(feature = "accesskit")]
    pub(crate) accesskit_adapter: Option<Adapter>,
    pub(crate) runtime: CraftRuntimeHandle,
    pub(crate) modifiers: Modifiers,
    pub(crate) last_frame_time: time::Instant,
    pub redraw_flags: RedrawFlags,

    pub(crate) render_list: RenderList,

    pub(crate) previous_animation_flags: AnimationFlags,

    pub(crate) focus: Option<Weak<RefCell<dyn Element>>>,
}

#[derive(Debug)]
pub struct RedrawFlags {
    rebuild_layout: bool,
}

impl RedrawFlags {
    pub fn new(rebuild_layout: bool) -> Self {
        Self { rebuild_layout }
    }

    pub fn should_rebuild_layout(&self) -> bool {
        self.rebuild_layout
    }
}

impl App {
    pub fn on_close_requested(&mut self) {
        info!("Craft application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        self.window_context.scale_factor = scale_factor;
        self.on_resize(self.window.as_ref().unwrap().inner_size());
    }

    pub fn on_process_user_events(&mut self) {}

    #[allow(unused_variables)]
    pub fn on_resume(&mut self, window: Arc<Window>, renderer: RendererBox, event_loop: &ActiveEventLoop) {
        window.set_ime_allowed(true);

        self.setup_text_context();
        self.renderer = Some(renderer);

        self.window = Some(window.clone());

        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let action_handler = CraftAccessHandler {
            runtime_handle: self.runtime.clone(),
            app_sender: self.app_sender.clone(),
        };
        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let deactivation_handler = CraftDeactivationHandler::new();

        let scale_factor = window.scale_factor();

        self.window = Some(window.clone());
        self.window_context.scale_factor = scale_factor;
        self.on_resize(window.inner_size());
        let tree_update = self.on_request_redraw();

        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let craft_activation_handler = CraftActivationHandler::new(tree_update);

        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        {
            self.accesskit_adapter = Some(Adapter::with_direct_handlers(
                event_loop,
                &window,
                craft_activation_handler,
                #[cfg(feature = "accesskit")]
                action_handler,
                deactivation_handler,
            ));
        }

        window.set_visible(true);
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
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
            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            let mut text_context = TextContext::new();
            #[cfg(all(not(target_arch = "wasm32"), feature = "system_fonts"))]
            let text_context = TextContext::new();

            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            {
                let regular = include_bytes!("../../../fonts/Roboto-Regular.ttf");
                let bold = include_bytes!("../../../fonts/Roboto-Bold.ttf");
                let semi_bold = include_bytes!("../../../fonts/Roboto-SemiBold.ttf");
                let medium = include_bytes!("../../../fonts/Roboto-Medium.ttf");

                fn register_and_append(font_data: &'static [u8], text_context: &mut TextContext) {
                    let blob = peniko::Blob::new(Arc::new(font_data));
                    let fonts = text_context.font_context.collection.register_fonts(blob, None);

                    // Register all the Roboto families under parley::GenericFamily::SystemUi.
                    // This will become the fallback font for platforms like WASM.
                    text_context
                        .font_context
                        .collection
                        .append_generic_families(parley::GenericFamily::SystemUi, fonts.iter().map(|f| f.0));
                }

                register_and_append(regular, &mut text_context);
                register_and_append(bold, &mut text_context);
                register_and_append(semi_bold, &mut text_context);
                register_and_append(medium, &mut text_context);
            }

            self.text_context = Some(text_context);
        }
    }

    /// Updates the view by applying the latest changes to the reactive tree.
    pub(crate) fn update_view(&mut self) {}

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(feature = "accesskit")]
    pub fn on_request_redraw(&mut self) -> Option<TreeUpdate> {
        self.on_request_redraw_internal();
        self.window.as_ref()?;
        let window = self.window.as_mut().unwrap().clone();

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

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(not(feature = "accesskit"))]
    pub fn on_request_redraw(&mut self) {
        self.on_request_redraw_internal();
    }

    //#[cfg(not(feature = "accesskit"))]

    fn on_request_redraw_internal(&mut self) {
        if self.window.is_none() {
            return;
        }

        self.update_resources();

        let now = time::Instant::now();
        let delta_time = now - self.last_frame_time;
        self.last_frame_time = now;

        let surface_size = self.window_context.window_size();

        self.update_view();

        let root_size = surface_size;

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().surface_set_clear_color(Color::WHITE);
        }

        let layout_origin = Point::new(0.0, 0.0);

        {
            if self.redraw_flags.should_rebuild_layout() {
                self.layout_tree(
                    root_size,
                    layout_origin,
                    self.window_context.effective_scale_factor(),
                    self.window_context.mouse_position,
                );
            }

            self.animate_tree(&delta_time, layout_origin, root_size);

            if self.renderer.is_some() {
                self.draw_reactive_tree(self.window_context.mouse_position, self.window.clone());
            }
        }

        {
            let span = span!(Level::INFO, "renderer_submit");
            let _enter = span.enter();

            if self.renderer.is_some() {
                self.renderer.as_mut().unwrap().submit(self.resource_manager.clone());
            }
        }

        if let Some(window) = &self.window {
            self.window_context.apply_requests(window);
            self.window_context.reset();
        }

        self.on_process_user_events();

        self.view_introspection();
    }

    pub fn on_pointer_scroll(&mut self, pointer_scroll_update: PointerScrollEvent) {
        if self.modifiers.ctrl() && pointer_scroll_update.pointer.pointer_type == ui_events::pointer::PointerType::Mouse
        {
            let y: f32 = match pointer_scroll_update.delta {
                ScrollDelta::PageDelta(_, y) => y,
                ScrollDelta::LineDelta(_, y) => y,
                PixelDelta(physical) => physical.y as f32,
            };
            if y < 0.0 {
                self.window_context.zoom_out();
            } else {
                self.window_context.zoom_in();
            }
            self.request_redraw(RedrawFlags::new(true));
            return;
        }

        let event = CraftMessage::PointerScroll(pointer_scroll_update);
        let message = event;

        self.dispatch_event(&message, false);
        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_pointer_button(&mut self, pointer_event: PointerButtonEvent, is_up: bool) {
        let mut pointer_event = pointer_event;
        let zoom = self.window_context.zoom_factor;
        pointer_event.state.position.x /= zoom;
        pointer_event.state.position.y /= zoom;

        let cursor_position = pointer_event.state.position;

        let event = if is_up {
            CraftMessage::PointerButtonUp(pointer_event)
        } else {
            CraftMessage::PointerButtonDown(pointer_event)
        };
        let message = event;
        self.window_context.mouse_position = Some(Point::new(cursor_position.x, cursor_position.y));

        self.dispatch_event(&message, true);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_pointer_moved(&mut self, mouse_moved: PointerUpdate) {
        let mut mouse_moved = mouse_moved;
        let zoom = self.window_context.zoom_factor;
        mouse_moved.current.position.x /= zoom;
        mouse_moved.current.position.y /= zoom;

        self.window_context.mouse_position = Some(mouse_moved.current.logical_point());

        let message = CraftMessage::PointerMovedEvent(mouse_moved);

        self.dispatch_event(&message, true);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_ime(&mut self, ime: Ime) {
        let event = CraftMessage::ImeEvent(ime);
        let message = event;

        self.dispatch_event(&message, false);

        self.request_redraw(RedrawFlags::new(true));
    }

    /// Dispatch messages to the reactive tree.
    fn dispatch_event(&mut self, message: &CraftMessage, _is_style: bool) {
        self.event_dispatcher.dispatch_event(
            message,
            self.window_context.mouse_position,
            self.root.clone(),
            &mut self.text_context,
        );
        self.window.clone().unwrap().request_redraw();
    }

    pub fn on_keyboard_input(&mut self, keyboard_input: KeyboardEvent) {
        self.modifiers = keyboard_input.modifiers;
        if keyboard_input.key == ui_events::keyboard::Key::Named(NamedKey::Control) && keyboard_input.state.is_up() {
            self.modifiers.set(Modifiers::CONTROL, false);
        }
        if keyboard_input.modifiers.ctrl() {
            if keyboard_input.key == ui_events::keyboard::Key::Character("=".to_string()) {
                self.window_context.zoom_in();
                self.request_redraw(RedrawFlags::new(true));
                return;
            } else if keyboard_input.key == ui_events::keyboard::Key::Character("-".to_string()) {
                self.window_context.zoom_out();
                self.request_redraw(RedrawFlags::new(true));
                return;
            }
        }

        let keyboard_event = CraftMessage::KeyboardInputEvent(keyboard_input.clone());
        let message = keyboard_event;

        self.dispatch_event(&message, false);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_resource_event(&mut self, resource_event: ResourceEvent) {
        match resource_event {
            ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                   in_progress.retain_mut(|(resource, _resource_type)| *resource != resource_identifier);
                });
                if let Some(_text_context) = self.text_context.as_mut()
                    && resource_type == ResourceType::Font
                    && resource.data().is_some()
                {
                    // Todo: Load the font into the text context.
                    self.resource_manager.insert(resource_identifier.clone(), Arc::new(resource));
                    self.reload_fonts = true;
                } else if resource_type == ResourceType::Image || resource_type == ResourceType::TinyVg {
                    self.resource_manager.insert(resource_identifier, Arc::new(resource));
                }
            }
            ResourceEvent::UnLoaded(_) => {}
        }
    }

    fn view_introspection(&mut self) {}

    fn request_redraw(&mut self, redraw_flags: RedrawFlags) {
        self.redraw_flags = redraw_flags;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// "Animates" a tree by calling `on_animation_frame` and changing an element's styles.
    fn animate_tree(&mut self, delta_time: &Duration, layout_origin: Point, viewport_size: LogicalSize<f32>) {
        let span = span!(Level::INFO, "animate_tree");
        let _enter = span.enter();

        let old_has_active_animation = self.previous_animation_flags.has_active_animation();
        let root_element = self.root.clone();

        // Damage track across recursive calls to `on_animation_frame`.
        let mut animation_flags = AnimationFlags::default();
        root_element.borrow_mut().on_animation_frame(&mut animation_flags, *delta_time);
        self.previous_animation_flags = animation_flags;

        // Perform a relayout if an animation used any layout effecting style property.
        if animation_flags.needs_relayout() || old_has_active_animation {
            root_element.borrow_mut().reset_layout_item();

            self.layout_tree(
                viewport_size,
                layout_origin,
                self.window_context.effective_scale_factor(),
                self.window_context.mouse_position,
            );
        }

        // Request a redraw if there is at least one animation playing.
        // ControlFlow::Poll is set in `about_to_wait`.
        if animation_flags.has_active_animation() || old_has_active_animation {
            // Winit does not guarantee when a redraw event will happen, but that should be fine, at worst we redraw an extra time.
            self.request_redraw(RedrawFlags::new(old_has_active_animation));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_tree(
        &mut self,
        viewport_size: LogicalSize<f32>,
        origin: Point,
        scale_factor: f64,
        mouse_position: Option<Point>,
    ) {
        let root_element = self.root.clone();
        style_root_element(root_element.borrow_mut().deref_mut(), viewport_size);
        let text_context = self.text_context.as_mut().unwrap();

        {
            let span = span!(Level::INFO, "layout");
            let _enter = span.enter();
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                layout(
                    taffy_tree,
                    root_element,
                    viewport_size,
                    text_context,
                    origin,
                    self.resource_manager.clone(),
                    scale_factor,
                    mouse_position,
                )
            });
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_reactive_tree(&mut self, mouse_position: Option<Point>, window: Option<Arc<Window>>) {
        let text_context = self.text_context.as_mut().unwrap();
        {
            let span = span!(Level::INFO, "render");
            let _enter = span.enter();
            self.render_list.clear();
            let scale_factor = self.window_context.effective_scale_factor();

            let renderer = self.renderer.as_mut().unwrap();

            self.root.borrow_mut().draw(&mut self.render_list, text_context, mouse_position, window, scale_factor);

            renderer.sort_and_cull_render_list(&mut self.render_list);

            let window = Rectangle {
                x: 0.0,
                y: 0.0,
                width: renderer.surface_width(),
                height: renderer.surface_height(),
            };
            renderer.prepare_render_list(&mut self.render_list, self.resource_manager.clone(), window);
        }
    }

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(&mut self) -> TreeUpdate {
        let tree = accesskit::Tree {
            root: accesskit::NodeId(0),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let focus_id = self.focus.clone().map(|node| node.upgrade().unwrap().borrow().id()).unwrap_or(0);
        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(focus_id),
        };

        self.root.borrow_mut().compute_accessibility_tree(
            &mut tree_update,
            None,
            self.window_context.effective_scale_factor(),
        );
        tree_update.nodes[0].1.set_role(Role::Window);

        tree_update
    }

    fn update_resources(&mut self) {
        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                for (resource, resource_type) in pending_resources.drain(..) {
                    if self.resource_manager.contains(&resource) || in_progress.contains(&(resource.clone(), resource_type)) {
                        continue;
                    }
                    self.resource_manager.async_download_resource_and_send_message_on_finish(self.app_sender.clone(), resource.clone(), resource_type);
                    in_progress.push_back((resource, resource_type));
                }
            });
        });
    }
}

fn style_root_element(root: &mut dyn Element, root_size: LogicalSize<f32>) {
    let is_user_root_height_auto = {
        let root_children = root.children();
        root_children[0].borrow().style().height().is_auto()
    };

    let style = root.style_mut();

    style.set_width(Unit::Px(root_size.width));
    style.set_wrap(Wrap::Wrap);
    style.set_display(Display::Block);

    if is_user_root_height_auto {
        style.set_height(Unit::Auto);
    } else {
        style.set_height(Unit::Px(root_size.height));
    }
}

#[allow(clippy::too_many_arguments)]
fn layout(
    taffy_tree: &mut TaffyTree<LayoutContext>,
    root_element: Rc<RefCell<dyn Element>>,
    window_size: LogicalSize<f32>,
    text_context: &mut TextContext,
    origin: Point,
    resource_manager: Arc<ResourceManager>,
    scale_factor: f64,
    pointer: Option<Point>,
) -> NodeId {
    let root_node = {
        let span = span!(Level::INFO, "compute layout(internal)");
        let _enter = span.enter();
        root_element.borrow_mut().compute_layout(taffy_tree, scale_factor);
        // There should usually be a layout node. If not, exiting early could also make sense.
        root_element.borrow()
            .element_data()
            .layout_item
            .taffy_node_id
            .expect("A root element must have a layout node.")
    };

    let available_space: taffy::Size<AvailableSpace> = taffy::Size {
        width: AvailableSpace::Definite(window_size.width),
        height: AvailableSpace::Definite(window_size.height),
    };

    {
        let span = span!(Level::INFO, "layout(taffy)");
        let _enter = span.enter();
        taffy_tree
            .compute_layout_with_measure(
                root_node,
                available_space,
                |known_dimensions, available_space, _node_id, node_context, style| {
                    measure_content(
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
    }

    let transform = Affine::IDENTITY;

    let mut layout_order: u32 = 0;
    {
        let span = span!(Level::INFO, "layout(apply)");
        let _enter = span.enter();
        root_element.borrow_mut().apply_layout(
            taffy_tree,
            root_node,
            origin,
            &mut layout_order,
            transform,
            pointer,
            text_context,
            None,
        );
    }

    SPATIAL_TREE.with_borrow_mut(|spatial_tree| {
        spatial_tree.commit();
    });

    root_node
}
