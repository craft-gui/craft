mod render_context;
pub mod image;
pub mod text;

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use glifo::GlyphRenderer;
use kurbo::{Affine, Stroke};
use peniko::kurbo::{Point, Shape};
use peniko::{BlendMode, ColorStop, ColorStops, Compose, Fill, InterpolationAlphaSpace, LinearGradientPosition, Mix};
use peniko::color::{DynamicColor, HueDirection};
use vello_common::filter_effects::{Filter, FilterFunction};
use vello_common::paint::{ImageId, PaintType};
use vello_common::{kurbo, peniko};
use vello_common::{peniko::Gradient, peniko::GradientKind};
use vello_common::color::ColorSpaceTag;
use vello_hybrid::{RenderSize, Renderer as VelloRenderer, Resources, Scene, TextureBindings};

use wgpu::CommandEncoder;
use wgpu::{CurrentSurfaceTexture, TextureFormat};

use winit::window::Window;
use craft_primitives::brush::Brush;
use craft_primitives::geometry::{Rectangle, TOLERANCE};
use craft_primitives::Color;
use craft_resource_manager::ResourceManager;
use crate::helpers::brush_to_paint;
use crate::render_command::{BoxShadowCmd, DrawCircleCmd, DrawCircleOutlineCmd, DrawRectCmd, DrawRectOutlineCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
use crate::renderer::Renderer;
use crate::resource_mapper::{RendererResourceId, ResourceMapper};
use crate::sort_commands::SortedCommands;
use render_context::{create_vello_renderer, DeviceHandle, RenderContext, RenderSurface};
use crate::RenderCommand;
use image::{draw_image, upload_image};
use text::draw_text;

pub struct ActiveRenderState {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'static>,
    window_width: f32,
    window_height: f32,
}

// This enum is only a few hundred bytes.
#[allow(clippy::large_enum_variant)]
enum RenderState {
    Active(ActiveRenderState),
    Suspended,
}

pub struct VelloHybridRenderer {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<VelloRenderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc.)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,

    resources: Resources,
    resource_mapper: ResourceMapper,
    resources_seen: HashSet<RendererResourceId>,

    window: Arc<Window>,

    texture_bindings: TextureBindings,

    render_list: RenderList,
}

impl Renderer for VelloHybridRenderer {
    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window_width,
            RenderState::Suspended => 0.0,
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window_height,
            RenderState::Suspended => 0.0,
        }
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };
        render_state.window_height = height;
        render_state.window_width = width;
        self.context
            .resize_surface(&mut render_state.surface, width as u32, height as u32);
        self.scene = Scene::new(width as u16, height as u16);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn render_list(&self) -> &RenderList {
        &self.render_list
    }

    fn render_list_mut(&mut self) -> &mut RenderList {
        &mut self.render_list
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(
        &mut self,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    ) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };

        self.resources_seen.clear();

        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        self.scene.set_transform(Affine::IDENTITY);

        // There is no way to clear the bg/clear color currently:
        draw_rect(
            &mut self.scene,
            &DrawRectCmd {
                rect: Rectangle::new(0.0, 0.0, width as f32, height as f32),
                brush: Brush::Color(self.surface_clear_color),
                transform: Affine::IDENTITY,
            }
        );

        let renderer = self.renderers[surface.dev_id].as_mut().unwrap();
        let device_handle = &self.context.devices[surface.dev_id];
        let mut encoder = device_handle
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blit Textures onto a Texture Atlas Encoder"),
            });

        let render_list = &self.render_list;
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {

            match command {
                RenderCommand::DrawCircle(cmd) => draw_circle(&mut self.scene, cmd),
                RenderCommand::DrawCircleOutline(cmd) => draw_circle_outline(&mut self.scene, cmd),
                RenderCommand::DrawRect(cmd) => draw_rect(&mut self.scene, cmd),
                RenderCommand::DrawRectOutline(cmd) => draw_rect_outline(&mut self.scene, cmd),
                RenderCommand::DrawImage(cmd) => {
                    if let Some(resource_id) = upload_image(
                        cmd,
                        resource_manager.clone(),
                        &mut self.resource_mapper,
                        &mut self.resources,
                        renderer,
                        &mut encoder,
                        device_handle,
                    ) {
                        draw_image(cmd, &mut self.scene, resource_manager.clone(), resource_id);
                    }

                    // Track the resources used.
                    if let Some(resource) = self.resource_mapper.get(&cmd.resource_id) {
                        self.resources_seen.insert(resource);
                    }
                }
                RenderCommand::DrawText(cmd) => {
                    draw_text(
                        cmd,
                        &mut self.scene,
                        &mut self.resources,
                        &window
                    );
                }
                RenderCommand::PushLayer(cmd) => {
                    push_layer(cmd, &mut self.scene);
                }
                RenderCommand::PopLayer => {
                    pop_layer(&mut self.scene);
                }
                RenderCommand::FillBezPath(cmd) => {
                    draw_filled_bez_path(cmd, &mut self.scene);
                }
                RenderCommand::StrokeBezPath(cmd) => {
                    draw_stroked_bez_path(cmd, &mut self.scene);
                }
                RenderCommand::StartOverlay => {}
                RenderCommand::EndOverlay => {}
                RenderCommand::BoxShadowCmd(cmd) => draw_box_shadow(&mut self.scene, cmd),
            }
        });

        let paint = self.scene.current_paint().clone();
        self.scene.set_paint(

            Gradient::new_linear(Point::new(20.0, 20.0), Point::new(220.0, 20.0))
                .with_hue_direction(HueDirection::Shorter)
                .with_extend(peniko::Extend::Pad)
                .with_interpolation_cs(ColorSpaceTag::Srgb)
                .with_interpolation_alpha_space(InterpolationAlphaSpace::Premultiplied)
                .with_stops(
                    [
                        ColorStop {
                            offset: 0.0,
                            color: DynamicColor::from(Color::from_rgb8(100, 100, 200)),
                        },
                        ColorStop {
                            offset: 1.0,
                            color: DynamicColor::from(Color::from_rgb8(255, 0, 0)),
                        },
                    ].as_slice()
                )
        );
        self.scene.fill_rect(&Rectangle::new(20.0, 20.0, 200.0, 100.0).to_kurbo());
        self.scene.set_paint(paint);


        VelloHybridRenderer::delete_unseen_resources(
            &mut self.resources_seen,
            renderer,
            &mut encoder,
            device_handle,
            &mut self.resources,
            &mut self.resource_mapper
        );

        device_handle.queue.submit([encoder.finish()]);
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("Todo: Handle a suspended render state."),
        };

        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let _width = surface.config.width;
        let _height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = match surface.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Suboptimal(_) => {
                self.context.configure_surface(surface);
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Lost => panic!("Surface was lost"),
            CurrentSurfaceTexture::Validation => {
                panic!("Validation error getting surface")
            }
        };

        let render_size = RenderSize {
            width: surface.config.width,
            height: surface.config.height,
        };

        let mut encoder = device_handle
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Vello Render to Surface pass"),
            });
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render(
                &self.scene,
                &mut self.resources,
                &device_handle.device,
                &device_handle.queue,
                &mut encoder,
                &render_size,
                &texture_view,
                &self.texture_bindings,
            )
            .unwrap();

        device_handle.queue.submit([encoder.finish()]);

        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}

impl VelloHybridRenderer {
    pub(crate) async fn new(window: Arc<Window>) -> VelloHybridRenderer {
        // Create a vello Surface
        let surface_size = window.inner_size();

        let width = surface_size.width.max(1);
        let height = surface_size.height.max(1);

        let mut vello_renderer = VelloHybridRenderer {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended,
            scene: Scene::new(width as u16, height as u16),
            surface_clear_color: Color::WHITE,
            resources: Resources::new(),
            resource_mapper: ResourceMapper::new(),
            resources_seen: HashSet::with_capacity(20),
            window: window.clone(),
            texture_bindings: Default::default(),
            render_list: Default::default(),
        };

        let surface = vello_renderer
            .context
            .create_surface(
                window.clone(),
                width,
                height,
                wgpu::PresentMode::AutoVsync,
                #[cfg(feature = "vello_hybrid_renderer_webgl")]
                TextureFormat::Rgba8Unorm,
                #[cfg(not(feature = "vello_hybrid_renderer_webgl"))]
                TextureFormat::Bgra8Unorm,
            )
            .await;

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer
            .renderers
            .resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState {
            surface,
            window_width: width as f32,
            window_height: height as f32,
        });

        vello_renderer
    }

    pub(crate) fn delete_unseen_resources(resources_seen: &mut HashSet<RendererResourceId>,
                                          renderer: &mut VelloRenderer,
                                          encoder: &mut CommandEncoder,
                                          device_handle: &DeviceHandle,
                                          resources: &mut Resources,
                                          resource_mapper: &mut ResourceMapper
    ) {
        resource_mapper.resources.retain(|_key, value| {
            if resources_seen.contains(&value) {
                true
            } else {
                renderer.destroy_image(
                    resources,
                    &device_handle.device,
                    &device_handle.queue,
                    encoder,
                    ImageId::new(value.0 as u32),
                );

                false
            }
        });
    }
}

fn draw_circle(scene: &mut Scene, cmd: &DrawCircleCmd) {
    scene.set_transform(cmd.transform);
    scene.set_paint(brush_to_paint(&cmd.brush));
    scene.fill_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
}

fn draw_rect(scene: &mut Scene, cmd: &DrawRectCmd) {
    scene.set_transform(cmd.transform);
    scene.set_paint(brush_to_paint(&cmd.brush));
    scene.fill_rect(&cmd.rect.to_kurbo());
}

fn draw_box_shadow(scene: &mut Scene, cmd: &BoxShadowCmd) {
    let radius = cmd.box_shadow.blur_radius / 2.0;
    let filter = Some(Filter::from_function(FilterFunction::Blur {
        radius: cmd.box_shadow.blur_radius as f32,
    }));

    if cmd.box_shadow.inset {
        scene.set_transform(cmd.transform);

        let mut clip_path = kurbo::BezPath::new();
        let outline_rect = cmd.box_shadow.border_box.expand((radius * 3.0) as f32).to_kurbo();
        clip_path.extend(&outline_rect.to_path(0.1));
        clip_path.extend(&cmd.box_shadow.path);
        scene.push_layer(Some(&cmd.box_shadow.outline), None, None, None, filter);
        scene.set_fill_rule(Fill::EvenOdd);
        scene.set_paint(cmd.box_shadow.color);
        scene.fill_path(&clip_path);
        scene.pop_layer();
        scene.set_fill_rule(Fill::NonZero);
    } else {
        scene.push_layer(
            None,
            Some(BlendMode::new(Mix::Normal, Compose::SrcOver)),
            None,
            None,
            filter,
        );

        scene.set_transform(cmd.transform * Affine::translate(cmd.box_shadow.offset));
        scene.set_paint(cmd.box_shadow.color);
        scene.fill_path(&cmd.box_shadow.path);
        scene.set_transform(cmd.transform);

        scene.set_blend_mode(BlendMode::new(Mix::Normal, Compose::DestOut));
        scene.set_paint(Color::WHITE);
        scene.fill_path(&cmd.box_shadow.outline);
        scene.set_blend_mode(BlendMode::new(Mix::Normal, Compose::SrcOver));

        scene.pop_layer();
    }
}

fn draw_circle_outline(scene: &mut Scene, cmd: &DrawCircleOutlineCmd) {
    scene.set_transform(cmd.transform);
    scene.set_stroke(Stroke::new(cmd.thickness as f64));
    scene.set_paint(brush_to_paint(&cmd.outline_brush));
    scene.stroke_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
}

fn draw_rect_outline(scene: &mut Scene, cmd: &DrawRectOutlineCmd) {
    scene.set_transform(cmd.transform);
   scene.set_stroke(Stroke::new(cmd.thickness));
    scene.set_paint(brush_to_paint(&cmd.outline_brush));
   scene.stroke_rect(&cmd.rect.to_kurbo());
}

fn push_layer(cmd: &PushLayerCmd, scene: &mut Scene) {
   match cmd {
        PushLayerCmd::BezPath(path, transform) => {
            scene.set_transform(*transform);
            scene.push_layer(Some(path), None, None, None, None);
        },
        PushLayerCmd::Rect(rect, transform) => {
            scene.set_transform(*transform);
            let clip_path = &rect.to_kurbo().into_path(0.1);
            scene.push_layer(Some(clip_path), None, None, None, None);
        },
   };
}

fn pop_layer(scene: &mut Scene) {
    scene.pop_layer();
}

fn draw_filled_bez_path(cmd: &FillBezPathCmd, scene: &mut Scene) {
    scene.set_transform(cmd.transform);
    scene.set_paint(brush_to_paint(&cmd.brush));
    scene.fill_path(&cmd.path);
}

fn draw_stroked_bez_path(cmd: &StrokeBezPathCmd, scene: &mut Scene) {
    scene.set_transform(cmd.transform);
    scene.set_paint(brush_to_paint(&cmd.brush));
    scene.stroke_path(&cmd.path);
}
