pub mod application;
pub mod components;
pub mod elements;
//mod lib2;
pub mod renderer;
mod widget_id;

pub mod events;
pub mod reactive;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use cosmic_text::{FontSystem, SwashCache};
use std::sync::Arc;
use std::time;
use slotmap::{DefaultKey, SlotMap};
use taffy::NodeId;
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};
use crate::components::component::{ComponentOrElement, ComponentSpecification};
use crate::elements::component::{Component, default_update};

use crate::elements::container::Container;
use crate::elements::layout_context::{measure_content, LayoutContext};
use crate::elements::element::Element;
use crate::elements::style::{Style, Unit};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Renderer};
use crate::renderer::softbuffer::SoftwareRenderer;
use crate::renderer::wgpu::WgpuRenderer;
const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App {
    app: ComponentSpecification,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    renderer_context: Option<RenderContext>,
    element_tree: Option<Box<dyn Element>>,
    mouse_position: (f32, f32),
}



pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

struct OkuState {
    id: u64,
    rt: tokio::runtime::Runtime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<Window>>,
    app_to_winit_rx: mpsc::Receiver<(u64, InternalMessage)>,
    winit_to_app_tx: mpsc::Sender<(u64, bool, InternalMessage)>,
    oku_options: OkuOptions,
}

#[derive(Copy, Clone, Debug)]
struct MouseInput {
    device_id: DeviceId,
    state: ElementState,
    button: MouseButton,
}

#[derive(Copy, Clone, Debug)]
struct MouseMoved {
    device_id: DeviceId,
    position: (f64, f64),
}

enum InternalMessage {
    RequestRedraw,
    Close,
    Confirmation,
    Resume(Arc<Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    MouseInput(MouseInput),
    MouseMoved(MouseMoved),
}

#[derive(Default)]
pub struct OkuOptions {
    pub renderer: RendererType,
}

#[derive(Default)]
pub enum RendererType {
    Software,
    #[default]
    Wgpu,
}

pub fn oku_main(application: ComponentSpecification) {
    oku_main_with_options(application, None)
}

pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create runtime");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop");

    let (winit_to_app_tx, winit_to_app_rx) = mpsc::channel::<(u64, bool, InternalMessage)>(100);
    let (app_to_winit_tx, app_to_winit_rx) = mpsc::channel::<(u64, InternalMessage)>(100);

    rt.spawn(async move {
        async_main(application, winit_to_app_rx, app_to_winit_tx).await;
    });

    let mut app = OkuState {
        id: 0,
        rt,
        request_redraw: false,
        wait_cancelled: false,
        close_requested: false,
        window: None,
        app_to_winit_rx,
        winit_to_app_tx,
        oku_options: options.unwrap_or_default(),
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for OkuState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("oku");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let renderer: Box<dyn Renderer + Send> = match self.oku_options.renderer {
            RendererType::Software => Box::new(SoftwareRenderer::new(window.clone())) as Box<dyn Renderer + Send>,
            RendererType::Wgpu => Box::new(self.rt.block_on(async { WgpuRenderer::new(window.clone()).await })),
        };

        self.send_message(InternalMessage::Resume(window, Some(renderer)), true);
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.send_message(InternalMessage::Close, true);
                self.close_requested = true;
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let mouse_event = MouseInput {
                    device_id,
                    state,
                    button,
                };
                if let MouseButton::Left = button {
                    self.request_redraw = true;

                    if state == ElementState::Pressed {
                        self.send_message(InternalMessage::MouseInput(mouse_event), true);
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                self.send_message(
                    InternalMessage::MouseMoved(MouseMoved {
                        device_id,
                        position: (position.x, position.y),
                    }),
                    true,
                );
            }
            WindowEvent::Resized(new_size) => {
                self.send_message(InternalMessage::Resize(new_size), true);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                if let Key::Named(NamedKey::Escape) = key.as_ref() {
                    self.close_requested = true;
                }
            }
            WindowEvent::RedrawRequested => {
                self.send_message(InternalMessage::RequestRedraw, true);
                self.window.clone().unwrap().pre_present_notify();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            //self.window.as_ref().unwrap().request_redraw();
        }

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }

        if self.close_requested {
            event_loop.exit();
        }
    }
}

impl OkuState {
    fn send_message(&mut self, message: InternalMessage, wait_for_response: bool) {
        let id = self.id;
        self.rt.block_on(async {
            self.winit_to_app_tx.send((id, wait_for_response, message)).await.expect("send failed");
            if wait_for_response {
                if let Some((id, InternalMessage::Confirmation)) = self.app_to_winit_rx.recv().await {
                    assert_eq!(id, self.id, "Expected response message with id {}", self.id);
                } else {
                    panic!("Expected response message");
                }
            }
        });
        self.id += 1;
    }
}

async fn send_response(id: u64, wait_for_response: bool, tx: &mpsc::Sender<(u64, InternalMessage)>) {
    if wait_for_response {
        tx.send((id, InternalMessage::Confirmation)).await.expect("send failed");
    }
}
use crate::events::{ClickMessage};
use crate::widget_id::{create_unique_widget_id, reset_unique_widget_id};

async fn async_main(application: ComponentSpecification, mut rx: mpsc::Receiver<(u64, bool, InternalMessage)>, tx: mpsc::Sender<(u64, InternalMessage)>) {
    let mut app = Box::new(App {
        app: application,
        window: None,
        renderer: None,
        renderer_context: None,
        element_tree: None,
        mouse_position: (0.0, 0.0),
    });

    loop {
        if let Some((id, wait_for_response, msg)) = rx.recv().await {
            match msg {
                InternalMessage::RequestRedraw => {
                    let renderer = app.renderer.as_mut().unwrap();

                    reset_unique_widget_id();
                    
                    renderer.surface_set_clear_color(Color::new_from_rgba_u8(255, 255, 255, 255));

                    let mut elements: SlotMap<DefaultKey, (Box<dyn Element>, Vec<DefaultKey>, Option<DefaultKey>)> = SlotMap::new();

                    let mut window_element = Container::new().background(Color::new_from_rgba_u8(0, 0, 255, 255));
                    *window_element.id_mut() = 9999;

                    let window_element = window_element.width(Unit::Px(renderer.surface_width()));

                    let root: DefaultKey = elements.insert((Box::new(window_element), vec![], None));

                    {
                        let mut to_visit: Vec<(Rc<RefCell<ComponentSpecification>>, DefaultKey)> = vec![(
                            Rc::new(RefCell::new(app.app.clone())), root.clone()
                        )];

                        while let Some(component) = to_visit.pop() {
                            let parent = component.1;
                            let component = component.0;

                            // Ignore the key and props for now... set key = None and props = None.
                            //let new_element = (component.borrow().component)(None, None);

                            // Build either a component or an element.

                            let children = component.borrow().children.clone();
                            match &component.borrow().component {
                                ComponentOrElement::Element(element) => {
                                    let new_element = elements.insert((element.clone(), vec![], Some(parent.clone())));
                                    elements[parent].1.push(new_element.clone());
                                    for child in children {
                                        println!("adding child with parent: {:?}", new_element);
                                        to_visit.push((Rc::new(RefCell::new(child)), new_element.clone()));
                                    }
                                },
                                ComponentOrElement::ComponentSpec(component_spec) => {
                                    let next_component_spec = Rc::new(RefCell::new(component_spec(None, None, children)));
                                    to_visit.push((next_component_spec, parent.clone()));
                                }
                            };
                        }
                    }

                    let mut levels: Vec<Vec<DefaultKey>> = vec![];

                    // Level order traversal
                    let mut to_visit: VecDeque<DefaultKey> = VecDeque::new();
                    to_visit.push_back(root.clone());
                    let mut level = 0;

                    while !to_visit.is_empty() {
                        let mut next_level: Vec<DefaultKey> = vec![];
                        let to_pop = to_visit.len();
                        for _ in 0..to_pop {
                            let node = to_visit.pop_front().unwrap();
                            let children = elements[node].1.clone();
                            next_level.push(node.clone());
                            for child in children {
                                to_visit.push_back(child);
                            }
                        }
                        levels.push(next_level);
                        level += 1;
                    }

                    for (levelno, level) in levels.iter().enumerate() {
                        for node in level {
                            let element = elements.get_mut(*node).unwrap().0.clone();
                            let mut parent_id = 0;
                            if let Some(parent) = elements.get(*node).unwrap().2 {
                                parent_id = elements[parent].0.id();
                            }
                            println!("level {} element: {} {} parent {}", levelno, element.name(), element.id(), parent_id);
                        }
                    }

                    println!("levels: {:?}", level);

                    for level in levels.iter().rev() {
                        for node in level {
                            let node = elements.get(*node).unwrap().clone();
                            if let Some(parent) = node.2 {
                                let parent = elements.get_mut(parent).unwrap();
                                parent.0.children_mut().push(node.0);
                            }
                        }
                    }

                    let mut root = elements.get_mut(root).unwrap().0.clone();

                    for child in root.children() {
                        for child in child.children() {
                            println!("child_: {}", child.name());
                        }
                    }

                    let computed_style = root.computed_style_mut();

                    // The root element should be 100% window width if the width is not already set.
                    if computed_style.width.is_auto() {
                        root.computed_style_mut().width = Unit::Px(renderer.surface_width());
                    }

                    let mut window_element: Box<dyn Element> = root;
                    window_element = layout(renderer.surface_width(), renderer.surface_height(), app.renderer_context.as_mut().unwrap(), &mut window_element);

                    
                    window_element.draw(renderer, app.renderer_context.as_mut().unwrap());
                    
                    app.element_tree = Some(window_element);

                    renderer.submit();
                    app.element_tree.as_ref().unwrap().clone_box().print_tree();
                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::Close => {
                    send_response(id, wait_for_response, &tx).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    if app.element_tree.is_none() {
                        reset_unique_widget_id();
                        //let new_view = app.app.view();
                        //app.element_tree = Some(new_view);
                    }

                    if app.renderer_context.is_none() {
                        let font_system = FontSystem::new();
                        let swash_cache = SwashCache::new();
                        let renderer_context = RenderContext {
                            font_system,
                            swash_cache,
                        };
                        app.renderer_context = Some(renderer_context);
                    }

                    app.window = Some(window.clone());
                    app.renderer = renderer;

                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::Resize(new_size) => {
                    let renderer = app.renderer.as_mut().unwrap();
                    renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);

                    // On macOS the window needs to be redrawn manually after resizing
                    app.window.as_ref().unwrap().request_redraw();

                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::MouseInput(mouse_input) => {
                    let root = app.element_tree.clone();
                    let mut to_visit = Vec::<Box<dyn Element>>::new();
                    let mut traversal_history = Vec::<Box<dyn Element>>::new();
                    to_visit.push(root.clone().unwrap());
                    traversal_history.push(root.unwrap());

                    while let Some(element) = to_visit.pop() {
                        for child in element.children() {
                            to_visit.push(child.clone());
                            traversal_history.push(child.clone());
                        }
                    }

                    for element in traversal_history.iter().rev() {
                        let in_bounds = element.in_bounds(app.mouse_position.0, app.mouse_position.1);
                        if !in_bounds {
                            continue;
                        }

                        /*let mut ch = Runtime::get_click_handler(0).unwrap();
                        let res = ch((2, 2));
                        Runtime::set_click_handler(0, ch);*/

                        /*match element {
                            Element::Component(component) => {
                                let update = component.update.clone();

                                let oku_message = events::Message::OkuMessage(
                                  events::OkuEvent::Click(ClickMessage {
                                      mouse_input,
                                      x: app.mouse_position.0 as f64,
                                      y: app.mouse_position.1 as f64,
                                  })
                                );

                                update(oku_message, Box::new(4), component.id());

                            }
                            _ => {}
                        }*/
                        
                       app.window.as_ref().unwrap().request_redraw();

                        /*if let EventResult::Stop = res {
                            break;
                        }*/
                    }

                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::MouseMoved(mouse_moved) => {
                    app.mouse_position = (mouse_moved.position.0 as f32, mouse_moved.position.1 as f32);
                    send_response(id, wait_for_response, &tx).await;
                }
            }
        }
    }
}

fn layout(_window_width: f32, _window_height: f32, render_context: &mut RenderContext, root_element: &mut Box<dyn Element>) -> Box<dyn Element> {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, &mut render_context.font_system);

    taffy_tree
        .compute_layout_with_measure(root_node, taffy::Size::max_content(), |known_dimensions, available_space, _node_id, node_context, style| measure_content(known_dimensions, available_space, node_context, &mut render_context.font_system))
        .unwrap();

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0);
    
    root_element.clone()
}
