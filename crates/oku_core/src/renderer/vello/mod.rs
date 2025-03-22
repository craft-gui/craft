mod image_adapter;
mod cosmic_adapter;

use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, Renderer, TextScroll};
use crate::renderer::vello::cosmic_adapter::CosmicFontBlobAdapter;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo::BezPath;
use peniko::Font;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "wgpu_renderer")]
use lyon::path::Path;
use tokio::sync::RwLockReadGuard;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{Glyph, Scene};
use vello::{kurbo, peniko, AaConfig, RendererOptions};
use winit::window::Window;
use crate::renderer::text;
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
    vello_fonts: HashMap<cosmic_text::fontdb::ID, Font>,
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
            vello_fonts: HashMap::new(),
        };

        // Create a vello Surface
        let surface_size = window.surface_size();

        let surface = vello_renderer
            .context
            .create_surface(
                window.clone(),
                surface_size.width,
                surface_size.height,
                wgpu::PresentMode::AutoVsync,
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
        vello_fonts: &HashMap<cosmic_text::fontdb::ID, Font>,
        scene: &mut Scene,
        resource_manager: &RwLockReadGuard<ResourceManager>,
        _font_system: &mut FontSystem,
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
                RenderCommand::DrawText(rect, component_id, fill_color, text_scroll) => {
                    let text_transform = Affine::translate((rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));


                    if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextInputState>()
                    {
                        let editor = &text_context.editor;
                        let buffer = &text_context.get_last_cache_entry().buffer;

                        let buffer_glyphs = text::create_glyphs_for_editor(
                            buffer,
                            editor,
                            fill_color,
                            Color::from_rgb8(0, 0, 0),
                            Color::from_rgb8(0, 120, 215),
                            Color::from_rgb8(255, 255, 255),
                            text_scroll,
                        );

                        // Draw the Glyphs
                        for buffer_line in &buffer_glyphs.buffer_lines {
                            for glyph_highlight in &buffer_line.glyph_highlights {
                                scene.fill(
                                    Fill::NonZero,
                                    text_transform,
                                    buffer_glyphs.glyph_highlight_color,
                                    None,
                                    glyph_highlight,
                                );
                            }

                            if let Some(cursor) = &buffer_line.cursor {
                                scene.fill(
                                    Fill::NonZero,
                                    text_transform,
                                    buffer_glyphs.cursor_color,
                                    None,
                                    cursor,
                                );
                            }

                            for glyph_run in &buffer_line.glyph_runs {
                                let font = vello_fonts.get(&glyph_run.font).unwrap();
                                let glyph_color = glyph_run.glyph_color;
                                let glyphs = glyph_run.glyphs.clone();
                                scene
                                    .draw_glyphs(font)
                                    .font_size(buffer_glyphs.font_size)
                                    .brush(glyph_color)
                                    .transform(text_transform)
                                    .draw(Fill::NonZero, glyphs.into_iter().map(|glyph| Glyph {
                                        id: glyph.glyph_id as u32,
                                        x: glyph.x,
                                        y: glyph.y + glyph_run.line_y,
                                    }));
                            }
                        }
                    } else if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().data.downcast_ref::<TextState>()
                    {
                        let buffer = &text_context.get_last_cache_entry().buffer;

                        let buffer_glyphs = text::create_glyphs(buffer, fill_color, None, None);
                        // Draw the Glyphs
                        for buffer_line in &buffer_glyphs.buffer_lines {
                            for glyph_run in &buffer_line.glyph_runs {
                                let font = vello_fonts.get(&glyph_run.font).unwrap();
                                let glyph_color = glyph_run.glyph_color;
                                let glyphs = glyph_run.glyphs.clone();
                                scene
                                    .draw_glyphs(font)
                                    .font_size(buffer_glyphs.font_size)
                                    .brush(glyph_color)
                                    .transform(text_transform)
                                    .draw(Fill::NonZero, glyphs.into_iter().map(|glyph| Glyph {
                                        id: glyph.glyph_id as u32,
                                        x: glyph.x,
                                        y: glyph.y + glyph_run.line_y,
                                    }));
                            }
                        }
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };
                }
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

    fn load_font(&mut self, font_system: &mut FontSystem) {
        let font_faces: Vec<(cosmic_text::fontdb::ID, u32)> =
            font_system.db().faces().map(|face| (face.id, face.index)).collect();
        for (font_id, index) in font_faces {
            if let Some(font) = font_system.get_font(font_id) {
                let font_blob = Blob::new(Arc::new(CosmicFontBlobAdapter::new(font)));
                let vello_font = Font::new(font_blob, index);
                self.vello_fonts.insert(font_id, vello_font);
            }
        }
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, _rectangle: Rectangle, _outline_color: Color) {}

    fn fill_bez_path(&mut self, path: BezPath, color: Color) {
        self.render_commands.push(RenderCommand::FillBezPath(path, color));
    }

    #[cfg(feature = "wgpu_renderer")]
    fn fill_lyon_path(&mut self, _path: &Path, _color: Color) { }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color, text_scroll: Option<TextScroll>) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color, text_scroll));
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

    fn prepare(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        _font_system: &mut FontSystem,
        element_state: &ElementStateStore) {
        VelloRenderer::prepare_with_render_commands(&self.vello_fonts, &mut self.scene, &resource_manager, _font_system, element_state, &mut self.render_commands);
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
                    base_color: self.surface_clear_color,
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
}
