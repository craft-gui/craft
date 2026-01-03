mod tinyvg;

use std::any::Any;
use std::sync::Arc;

use craft_primitives::Color;
use craft_primitives::geometry::Rectangle;
use craft_resource_manager::ResourceManager;
use craft_resource_manager::resource::Resource;
use peniko::{BrushRef, ImageAlphaType};
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::{AaConfig, Error, Glyph, RendererOptions, Scene, kurbo, peniko};
use wgpu::util::TextureBlitter;
use wgpu::{Adapter, Device, Instance, Limits, MemoryHints, Queue, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture, Texture, TextureFormat, TextureView};
use winit::window::Window;

use crate::image_adapter::ImageAdapter;
use crate::renderer::{RenderCommand, RenderList, Renderer, SortedCommands, TextScroll};
use crate::text_renderer_data::TextRenderLine;
use crate::vello::tinyvg::draw_tiny_vg;

pub struct RenderSurface {
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub surface_texture: wgpu::Texture,
    pub surface_view: wgpu::TextureView,
}

impl RenderSurface {
    pub fn create_surface_textures(device: &Device, surface_width: u32, surface_height: u32) -> (Texture, TextureView) {
        let surface_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: surface_width,
                height: surface_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            format: TextureFormat::Rgba8Unorm,
            view_formats: &[],
        });
        let surface_texture_view = surface_texture.create_view(&wgpu::TextureViewDescriptor::default());

        (surface_texture, surface_texture_view)
    }

    /// Returns the current swapchain `SurfaceTexture`.
    /// Unlike the cached textures that we store, this always fetches a fresh, up-to-date frame.
    pub fn get_swapchain_surface_texture(
        &mut self,
        device: &Device,
        surface_width: u32,
        surface_height: u32,
    ) -> SurfaceTexture {
        match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(err) => {
                match err {
                    SurfaceError::Timeout | SurfaceError::Outdated | SurfaceError::Lost => {
                        self.resize(device, surface_width, surface_height);
                    }
                    SurfaceError::OutOfMemory | SurfaceError::Other => {
                        panic!("Failed to acquire surface texture: {err:?}");
                    }
                }
                self.surface
                    .get_current_texture()
                    .expect("Failed to get surface texture after resize")
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn resize(&mut self, device: &Device, surface_width: u32, surface_height: u32) {
        self.surface_config.width = surface_width;
        self.surface_config.height = surface_height;
        let (surface_texture, surface_view) = Self::create_surface_textures(device, surface_width, surface_height);

        self.surface_texture = surface_texture;
        self.surface_view = surface_view;

        self.surface.configure(device, &self.surface_config);
    }

    pub fn new(
        device: &Device,
        adapter: &Adapter,
        surface: Surface<'static>,
        surface_width: u32,
        surface_height: u32,
    ) -> RenderSurface {
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .into_iter()
            .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
            .ok_or(Error::UnsupportedSurfaceFormat)
            .expect("Unsupported surface format.");

        let (surface_texture, surface_view) = Self::create_surface_textures(device, surface_width, surface_height);

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: surface_width,
            height: surface_height,
            present_mode: wgpu::PresentMode::Immediate,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(device, &surface_config);

        RenderSurface {
            surface,
            surface_config,
            surface_texture,
            surface_view,
        }
    }
}

pub struct VelloRenderer {
    pub device: Device,
    #[allow(dead_code)]
    pub adapter: Adapter,
    pub queue: Queue,
    #[allow(dead_code)]
    pub instance: Instance,
    pub render_surface: RenderSurface,
    pub texture_blitter: TextureBlitter,
    pub renderer: vello::Renderer,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
    pub surface_clear_color: Color,
    pub render_into_texture: bool,
}

fn create_vello_renderer(device: &Device) -> vello::Renderer {
    vello::Renderer::new(
        device,
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

fn new_instance() -> Instance {
    let backends = wgpu::Backends::from_env().unwrap_or_default();
    let flags = wgpu::InstanceFlags::from_build_config().with_env();
    let backend_options = wgpu::BackendOptions::from_env_or_default();
    Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags,
        memory_budget_thresholds: Default::default(),
        backend_options,
    })
}

async fn new_device(instance: &Instance, surface: &Surface<'_>) -> (Device, Queue, Adapter) {
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(instance, Some(surface))
        .await
        .expect("Failed to create an adapter.");
    let features = adapter.features();
    let limits = Limits::default();
    let maybe_features = wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: features & maybe_features,
            required_limits: limits,
            memory_hints: MemoryHints::default(),
            trace: Default::default(),
        })
        .await
        .expect("Failed to create device.");

    (device, queue, adapter)
}

impl VelloRenderer {
    pub async fn new(window: Arc<Window>, render_into_texture: bool) -> VelloRenderer {
        let window_size = window.inner_size();

        let instance = new_instance();
        let surface = instance.create_surface(window).expect("Failed to create a surface.");
        let (device, queue, adapter) = new_device(&instance, &surface).await;
        let render_surface = RenderSurface::new(&device, &adapter, surface, window_size.width, window_size.height);

        VelloRenderer {
            texture_blitter: TextureBlitter::new(&device, render_surface.surface_config.format),
            render_surface,
            renderer: create_vello_renderer(&device),
            device,
            adapter,
            queue,
            instance,
            scene: Scene::new(),
            surface_clear_color: Color::WHITE,
            render_into_texture,
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

impl Renderer for VelloRenderer {
    fn surface_width(&self) -> f32 {
        self.render_surface.width() as f32
    }

    fn surface_height(&self) -> f32 {
        self.render_surface.height() as f32
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.render_surface.resize(&self.device, width as u32, height as u32);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare_render_list<'a>(
        &mut self,
        render_list: &mut RenderList,
        resource_manager: Arc<ResourceManager>,
        window: Rectangle,
    ) {
        SortedCommands::draw(render_list, &render_list.overlay, &mut |command: &RenderCommand| {
            let scene = &mut self.scene;

            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(scene, *rectangle, *fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color, thickness) => {
                    self.scene.stroke(
                        &Stroke::new(*thickness),
                        Affine::IDENTITY,
                        outline_color,
                        None,
                        &rectangle.to_kurbo(),
                    );
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.get(resource_identifier);
                    if let Some(resource) = resource
                        && let Resource::Image(resource) = resource.as_ref()
                    {
                        let image = &resource.image;
                        let data = Arc::new(ImageAdapter::new(resource.clone()));
                        let blob = Blob::new(data);
                        let vello_image = vello::peniko::ImageData {
                            data: blob,
                            format: peniko::ImageFormat::Rgba8,
                            alpha_type: ImageAlphaType::Alpha,
                            width: image.width(),
                            height: image.height(),
                        };

                        let vello_image = vello::peniko::ImageBrush::new(vello_image);

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
                RenderCommand::DrawText(text_render, rect, text_scroll, show_cursor) => {
                    let text_transform =
                        Affine::default().with_translation(kurbo::Vec2::new(rect.x as f64, rect.y as f64));
                    let scroll = text_scroll.unwrap_or(TextScroll::default()).scroll_y;
                    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

                    let c = text_render.upgrade();
                    if c.is_none() {
                        return;
                    }
                    let c = c.unwrap();
                    let c = c.borrow();
                    let text_render = c.get_text_renderer().expect("Text render not found");

                    let cull_and_process = |process_line: &mut dyn FnMut(&TextRenderLine)| {
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
                            }

                            process_line(line);
                        }
                    };

                    cull_and_process(&mut |line: &TextRenderLine| {
                        for (background, color) in &line.backgrounds {
                            let background_rect = Rectangle {
                                x: background.x + rect.x,
                                y: -scroll + background.y + rect.y,
                                width: background.width,
                                height: background.height,
                            };
                            vello_draw_rect(scene, background_rect, *color);
                        }

                        for (selection, selection_color) in &line.selections {
                            let selection_rect = Rectangle {
                                x: selection.x + rect.x,
                                y: -scroll + selection.y + rect.y,
                                width: selection.width,
                                height: selection.height,
                            };
                            vello_draw_rect(scene, selection_rect, *selection_color);
                        }
                    });

                    cull_and_process(&mut |line: &TextRenderLine| {
                        for item in &line.items {
                            if let Some(underline) = &item.underline {
                                scene.stroke(
                                    &Stroke::new(underline.width.into()),
                                    text_transform,
                                    underline.brush.color,
                                    None,
                                    &underline.line,
                                );
                            }

                            scene
                                .draw_glyphs(&item.font)
                                .font_size(item.font_size)
                                .brush(BrushRef::Solid(
                                    text_render
                                        .override_brush
                                        .map(|b| b.color)
                                        .unwrap_or_else(|| item.brush.color),
                                ))
                                .transform(text_transform)
                                .glyph_transform(item.glyph_transform)
                                .draw(
                                    Fill::NonZero,
                                    item.glyphs.iter().map(|glyph| Glyph {
                                        id: glyph.id,
                                        x: glyph.x,
                                        y: glyph.y,
                                    }),
                                );
                        }
                    });

                    if *show_cursor && let Some((cursor, cursor_color)) = &text_render.cursor {
                        let cursor_rect = Rectangle {
                            x: cursor.x + rect.x,
                            y: -scroll + cursor.y + rect.y,
                            width: cursor.width,
                            height: cursor.height,
                        };
                        vello_draw_rect(scene, cursor_rect, *cursor_color);
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
        let width = self.render_surface.width();
        let height = self.render_surface.height();

        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self.render_surface.surface_view,
                &vello::RenderParams {
                    base_color: self.surface_clear_color,
                    width,
                    height,
                    antialiasing_method: if cfg!(any(target_os = "android", target_os = "ios")) {
                        AaConfig::Area
                    } else {
                        AaConfig::Msaa16
                    },
                },
            )
            .expect("failed to render to texture");

        if !self.render_into_texture {
            let swapchain_surface_texture =
                self.render_surface
                    .get_swapchain_surface_texture(&self.device, width, height);
            let swapchain_surface_texture_view = swapchain_surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surface Blit"),
            });

            self.texture_blitter.copy(
                &self.device,
                &mut encoder,
                &self.render_surface.surface_view,
                &swapchain_surface_texture_view,
            );
            self.queue.submit(Some(encoder.finish()));

            swapchain_surface_texture.present();
        }

        self.scene.reset();
    }
}
