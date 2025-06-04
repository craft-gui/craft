mod tinyvg;

use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::image_adapter::ImageAdapter;
use crate::renderer::renderer::{RenderCommand, RenderList, Renderer, SortedCommands, TextScroll};
use crate::renderer::vello::tinyvg::draw_tiny_vg;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::ResourceManager;
use peniko::BrushRef;
use std::sync::Arc;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{kurbo, peniko, AaConfig, RendererOptions};
use vello::{Glyph, Scene};
use winit::window::Window;

pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window_width: f32,
    window_height: f32,
}

// This enum is only a few hundred bytes.
#[allow(clippy::large_enum_variant)]
enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended,
}

pub struct VelloRenderer<'a> {
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
            pipeline_cache: None,
        },
    )
    .expect("Couldn't create renderer")
}

impl<'a> VelloRenderer<'a> {
    pub(crate) async fn new(window: Arc<Window>) -> VelloRenderer<'a> {
        let mut vello_renderer = VelloRenderer {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended,
            scene: Scene::new(),
            surface_clear_color: Color::WHITE,
        };

        // Create a vello Surface
        let surface_size = window.inner_size();

        let surface = vello_renderer
            .context
            .create_surface(window.clone(), surface_size.width, surface_size.height, wgpu::PresentMode::AutoVsync)
            .await
            .unwrap();

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState {
            surface,
            window_width: surface_size.width as f32,
            window_height: surface_size.height as f32,
        });

        vello_renderer
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
        render_state.window_width = width;
        render_state.window_height = height;
        self.context.resize_surface(&mut render_state.surface, width as u32, height as u32);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn prepare_render_list(
        &mut self,
        render_list: RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    ) {
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            let scene = &mut self.scene;

            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, *rectangle, *fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    self.scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, outline_color, None, &rectangle.to_kurbo());
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(resource_identifier);
                    if let Some(resource) = resource {
                        if let Resource::Image(resource) = resource.as_ref() {
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
                }
                RenderCommand::DrawText(text_render, rect, text_scroll, show_cursor) => {
                    let text_transform =
                        Affine::default().with_translation(kurbo::Vec2::new(rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let mut skip_remaining_lines = false;
                    let mut skip_line = false;
                    for line in &text_render.lines {
                        if skip_remaining_lines {
                            break;
                        }
                        if skip_line {
                            skip_line = false;
                            continue;
                        }
                        for item in &line.items {
                            if let Some(first_glyph) = item.glyphs.first() {
                                // Cull the selections vertically that are outside the window
                                let gy = first_glyph.y + rect.y - scroll;
                                if gy < window.y {
                                    skip_line = true;
                                    break;
                                } else if gy > (window.y + window.height) {
                                    skip_remaining_lines = true;
                                    break;
                                }
                            }

                            for selection in &line.selections {
                                let selection_rect = Rectangle {
                                    x: selection.x + rect.x,
                                    y: -scroll + selection.y + rect.y,
                                    width: selection.width,
                                    height: selection.height,
                                };
                                vello_draw_rect(scene, selection_rect, Color::from_rgb8(0, 120, 215));
                            }
                        }
                    }
                    skip_remaining_lines = false;
                    skip_line = false;
                    for line in &text_render.lines {
                        if skip_remaining_lines {
                            break;
                        }
                        if skip_line {
                            skip_line = false;
                            continue;
                        }
                        for item in &line.items {
                            if let Some(first_glyph) = item.glyphs.first() {
                                // Cull the glyphs vertically that are outside the window
                                let gy = first_glyph.y + rect.y - scroll;
                                if gy < window.y {
                                    skip_line = true;
                                    break;
                                } else if gy > (window.y + window.height) {
                                    skip_remaining_lines = true;
                                    break;
                                }
                            }

                            scene
                                .draw_glyphs(&item.font)
                                .font_size(item.font_size)
                                .brush(BrushRef::Solid(
                                    text_render.override_brush.map(|b| b.color).unwrap_or_else(|| item.brush.color),
                                ))
                                .transform(text_transform)
                                .glyph_transform(item.glyph_transform)
                                .draw(
                                    Fill::NonZero,
                                    item.glyphs.iter().map(|glyph| Glyph {
                                        id: glyph.id as u32,
                                        x: glyph.x,
                                        y: glyph.y,
                                    }),
                                );
                        }
                    }
                    if *show_cursor {
                        if let Some(cursor) = &text_render.cursor {
                            let cursor_rect = Rectangle {
                                x: cursor.x + rect.x,
                                y: -scroll + cursor.y + rect.y,
                                width: cursor.width,
                                height: cursor.height,
                            };
                            vello_draw_rect(scene, cursor_rect, Color::from_rgb8(0, 0, 0));
                        }
                    }
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, override_color) => {
                    draw_tiny_vg(
                        scene,
                        *rectangle,
                        resource_manager.clone(),
                        resource_identifier.clone(),
                        override_color,
                    );
                }
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
                RenderCommand::FillBezPath(path, brush) => {
                    scene.fill(Fill::NonZero, Affine::IDENTITY, brush, None, &path);
                }
                _ => {}
            }
        });
    }

    fn submit(&mut self, _resource_manager: Arc<ResourceManager>) {
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
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface.target_view,
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
        let mut encoder = device_handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
        });
        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        device_handle.queue.submit([encoder.finish()]);
        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}
