mod image_adapter;

use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, Renderer};
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use parley::{FontContext, Line, PositionedLayoutItem};
use peniko::kurbo::{BezPath, Stroke};
use peniko::Font;
use std::sync::Arc;
#[cfg(feature = "wgpu_renderer")]
use lyon::path::Path;
use tokio::sync::RwLockReadGuard;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::Scene;
use vello::{kurbo, peniko, AaConfig, RendererOptions};
use winit::window::Window;
use crate::renderer::vello::image_adapter::ImageAdapter;

pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<dyn Window>,
}

enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended,
}

pub struct VelloRenderer<'a> {
    render_commands: Vec<RenderCommand>,

    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<vello::Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState<'a>,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> vello::Renderer {
    vello::Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            // FIXME: Use msaa16 by default once https://github.com/linebender/vello/issues/723 is resolved.
            antialiasing_support: if cfg!(any(target_os = "android", target_os = "ios")) {
                vello::AaSupport {
                    area: true,
                    msaa8: false,
                    msaa16: false,
                }
            } else {
                vello::AaSupport {
                    area: false,
                    msaa8: false,
                    msaa16: true,
                }
            },
            num_init_threads: None,
        },
    )
    .expect("Couldn't create renderer")
}

impl<'a> VelloRenderer<'a> {
    pub(crate) async fn new(window: Arc<dyn Window>) -> VelloRenderer<'a> {
        let mut vello_renderer = VelloRenderer {
            render_commands: vec![],
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended,
            scene: Scene::new(),
            surface_clear_color: Color::WHITE,
        };

        // Create a vello Surface
        let surface_size = window.surface_size();

        let surface = vello_renderer
            .context
            .create_surface(
                window.clone(),
                surface_size.width,
                surface_size.height,
                vello::wgpu::PresentMode::AutoVsync,
            )
            .await
            .unwrap();

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState { window, surface });

        vello_renderer
    }

    fn prepare_with_render_commands(
        scene: &mut Scene,
        resource_manager: &RwLockReadGuard<ResourceManager>,
        _font_context: &mut FontContext,
        element_state: &ElementStateStore,
        render_commands: &mut Vec<RenderCommand>,
    ) {
        for command in render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(_rectangle, _outline_color) => {
                    // vello_draw_rect_outline(&mut self.scene, rectangle, outline_color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        let data = Arc::new(ImageAdapter::new(resource.clone()));
                        let blob = Blob::new(data);
                        let vello_image =
                            peniko::Image::new(blob, peniko::ImageFormat::Rgba8, image.width(), image.height());

                        let mut transform = Affine::IDENTITY;
                        transform =
                            transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            rectangle.width as f64 / image.width() as f64,
                            rectangle.height as f64 / image.height() as f64,
                        );

                        scene.draw_image(&vello_image, transform);
                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color) => {
                    let text_transform = Affine::translate((rect.x as f64, rect.y as f64));
                    
                    
                    
                    if let Some(text_state) = element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextState>() {
                        for line in text_state.layout.lines() {
                            for item in line.items() {
                                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                                    continue;
                                };
                                let style = glyph_run.style();
                                // We draw underlines under the text, then the strikethrough on top, following:
                                // https://drafts.csswg.org/css-text-decor/#painting-order
                                if let Some(underline) = &style.underline {
                                    let underline_brush = &style.brush;
                                    let run_metrics = glyph_run.run().metrics();
                                    let offset = match underline.offset {
                                        Some(offset) => offset,
                                        None => run_metrics.underline_offset,
                                    };
                                    let width = match underline.size {
                                        Some(size) => size,
                                        None => run_metrics.underline_size,
                                    };
                                    // The `offset` is the distance from the baseline to the top of the underline
                                    // so we move the line down by half the width
                                    // Remember that we are using a y-down coordinate system
                                    // If there's a custom width, because this is an underline, we want the custom
                                    // width to go down from the default expectation
                                    let y = glyph_run.baseline() - offset + width / 2.;

                                    let line = kurbo::Line::new(
                                        (glyph_run.offset() as f64, y as f64),
                                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                                    );
                                    scene.stroke(
                                        &Stroke::new(width.into()),
                                        text_transform,
                                        underline_brush,
                                        None,
                                        &line,
                                    );
                                }
                                let mut x = glyph_run.offset();
                                let y = glyph_run.baseline();
                                let run = glyph_run.run();
                                let font = run.font();
                                let font_size = run.font_size();
                                let synthesis = run.synthesis();
                                let glyph_xform = synthesis
                                    .skew()
                                    .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
                                scene
                                    .draw_glyphs(font)
                                    .brush(&style.brush)
                                    .hint(true)
                                    .transform(text_transform)
                                    .glyph_transform(glyph_xform)
                                    .font_size(font_size)
                                    .normalized_coords(run.normalized_coords())
                                    .draw(
                                        Fill::NonZero,
                                        glyph_run.glyphs().map(|glyph| {
                                            let gx = x + glyph.x;
                                            let gy = y - glyph.y;
                                            x += glyph.advance;
                                            vello::Glyph {
                                                id: glyph.id as _,
                                                x: gx,
                                                y: gy,
                                            }
                                        }),
                                    );
                                if let Some(strikethrough) = &style.strikethrough {
                                    let strikethrough_brush = &style.brush;
                                    let run_metrics = glyph_run.run().metrics();
                                    let offset = match strikethrough.offset {
                                        Some(offset) => offset,
                                        None => run_metrics.strikethrough_offset,
                                    };
                                    let width = match strikethrough.size {
                                        Some(size) => size,
                                        None => run_metrics.strikethrough_size,
                                    };
                                    // The `offset` is the distance from the baseline to the *top* of the strikethrough
                                    // so we calculate the middle y-position of the strikethrough based on the font's
                                    // standard strikethrough width.
                                    // Remember that we are using a y-down coordinate system
                                    let y = glyph_run.baseline() - offset + run_metrics.strikethrough_size / 2.;

                                    let line = kurbo::Line::new(
                                        (glyph_run.offset() as f64, y as f64),
                                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                                    );
                                    scene.stroke(
                                        &Stroke::new(width.into()),
                                        text_transform,
                                        strikethrough_brush,
                                        None,
                                        &line,
                                    );
                                }
                            }
                        }
                    }
                    
                    
                },
                /*RenderCommand::PushTransform(transform) => {
                    self.scene.push_transform(transform);
                },
                RenderCommand::PopTransform => {
                    self.scene.pop_transform();
                },*/
                RenderCommand::PushLayer(rect) => {
                    let clip = Rect::new(
                        rect.x as f64,
                        rect.y as f64,
                        (rect.x + rect.width) as f64,
                        (rect.y + rect.height) as f64,
                    );
                    scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                }
                RenderCommand::PopLayer => {
                    scene.pop_layer();
                }
                RenderCommand::FillBezPath(path, color) => {
                    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
                },
                #[cfg(feature = "wgpu_renderer")]
                RenderCommand::FillLyonPath(_, _) => {}
            }
        }
    }
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {
    let rect = Rect::new(
        rectangle.x as f64,
        rectangle.y as f64,
        (rectangle.x + rectangle.width) as f64,
        (rectangle.y + rectangle.height) as f64,
    );
    scene.fill(Fill::NonZero, Affine::IDENTITY, fill_color, None, &rect);
}

impl Renderer for VelloRenderer<'_> {
    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window.surface_size().width as f32,
            RenderState::Suspended => 0.0,
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => active_render_state.window.surface_size().height as f32,
            RenderState::Suspended => 0.0,
        }
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };

        self.context.resize_surface(&mut render_state.surface, width as u32, height as u32);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, _rectangle: Rectangle, _outline_color: Color) {}

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }
    
    fn load_font(&mut self, font_context: &mut FontContext) {
    
    }

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        _font_context: &mut FontContext,
        element_state: &ElementStateStore) {
        VelloRenderer::prepare_with_render_commands(&mut self.scene, &resource_manager, _font_context, element_state, &mut self.render_commands);
    }

    fn submit(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };

        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface.surface.get_current_texture().expect("failed to get surface texture");

        // Render to the surface's texture
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: self.surface_clear_color.into(),
                    width,
                    height,
                    // FIXME: Use msaa16 by default once https://github.com/linebender/vello/issues/723 is resolved.
                    antialiasing_method: if cfg!(any(target_os = "android", target_os = "ios")) {
                        AaConfig::Area
                    } else {
                        AaConfig::Msaa16
                    },
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }

    fn fill_bez_path(&mut self, path: BezPath, color: Color) {
        self.render_commands.push(RenderCommand::FillBezPath(path, color));
    }

    #[cfg(feature = "wgpu_renderer")]
    fn fill_lyon_path(&mut self, path: &Path, color: Color) {
    }
}
