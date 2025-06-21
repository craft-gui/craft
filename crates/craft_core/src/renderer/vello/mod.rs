mod tinyvg;

use std::any::Any;
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
use vello::{kurbo, peniko, AaConfig, Error, RendererOptions};
use vello::{Glyph, Scene};
use wgpu::{Adapter, Device, Instance, Limits, MemoryHints, Queue, Surface, SurfaceConfiguration, Texture, TextureFormat, TextureView};
use wgpu::util::TextureBlitter;
use winit::window::Window;
use crate::text::text_render_data::TextRenderLine;

pub struct VelloRenderer {
    device: Device,
    #[allow(dead_code)]
    adapter: Adapter,
    queue: Queue,
    #[allow(dead_code)]
    instance: Instance,
    surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,

    #[allow(dead_code)]
    surface_texture: Texture,
    surface_texture_view: TextureView,
    
    renderer: vello::Renderer,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,
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
    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags,
        backend_options,
    });
    instance
}

async fn new_device(instance: &Instance, surface: &Surface<'_>) -> (Device, Queue, Adapter) {
    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(instance, Some(surface))
            .await.expect("Failed to create an adapter.");
    let features = adapter.features();
    let limits = Limits::default();
    let maybe_features = wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features & maybe_features,
                required_limits: limits,
                memory_hints: MemoryHints::default(),
            },
            None,
        )
        .await
        .ok().expect("Failed to create device.");

    (
        device,
        queue,
        adapter,
    )
}

fn new_surface_texture(device: &Device, adapter: &Adapter,  surface: &Surface, surface_width: u32, surface_height: u32) -> (Texture, TextureView, SurfaceConfiguration) {
    let capabilities = surface.get_capabilities(adapter);
    let format = capabilities
        .formats
        .into_iter()
        .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
        .ok_or(Error::UnsupportedSurfaceFormat).expect("Unsupported surface format.");

    let config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: surface_width,
        height: surface_height,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };

    let target_texture = device.create_texture(&wgpu::TextureDescriptor {
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
    let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    surface.configure(device, &config);
    
    (target_texture, target_view, config)
}

impl VelloRenderer {
    
    pub async fn new(window: Arc<Window>) -> VelloRenderer {

        let window_size = window.inner_size();
        
        let instance = new_instance();
        let surface = instance.create_surface(window).expect("Failed to create a surface.");
        let (device, queue, adapter) = new_device(&instance, &surface).await;
        let (surface_texture, surface_texture_view, surface_config) = new_surface_texture(&device, &adapter, &surface, window_size.width, window_size.height);
        
        VelloRenderer {
            renderer: create_vello_renderer(&device),
            device,
            adapter,
            queue,
            instance,
            surface,
            surface_config,
            surface_texture,
            surface_texture_view,
            scene: Scene::new(),
            surface_clear_color: Color::WHITE,
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
        self.surface_config.width as f32
    }

    fn surface_height(&self) -> f32 {
        self.surface_config.height as f32
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        let (surface_texture, surface_texture_view, surface_config) = new_surface_texture(
            &self.device, &self.adapter, &self.surface,
            width as u32, height as u32
        );
        
        self.surface_texture = surface_texture;
        self.surface_texture_view = surface_texture_view;
        self.surface_config = surface_config;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
                        
                        for selection in &line.selections {
                            let selection_rect = Rectangle {
                                x: selection.x + rect.x,
                                y: -scroll + selection.y + rect.y,
                                width: selection.width,
                                height: selection.height,
                            };
                            vello_draw_rect(scene, selection_rect, Color::from_rgb8(0, 120, 215));
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
                    });
                    
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
        // Get the window size
        let width = self.surface_config.width;
        let height = self.surface_config.height;

        // Get the surface's texture
        let surface_texture = self.surface.get_current_texture().unwrap();

        // Render to the surface's texture
        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self.surface_texture_view,
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
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
        });
        
        
        let blitter = TextureBlitter::new(&self.device, self.surface_config.format);
        blitter.copy(
            &self.device,
            &mut encoder,
            &self.surface_texture_view,
            &surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        self.queue.submit([encoder.finish()]);
        
        // Queue the texture to be presented on the surface
        surface_texture.present();

        self.scene.reset();
    }
}
