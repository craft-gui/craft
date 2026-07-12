mod render_context;

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};

use craft_primitives::geometry::{Circle, Rectangle, TOLERANCE};
use craft_primitives::Color;

use craft_resource_manager::{ResourceId, ResourceManager};

use glifo::{DrawSink, Glyph};

use kurbo::{Affine, Stroke};

use peniko::kurbo::Shape;
use peniko::{BlendMode, Compose, Fill, Mix};

use vello_common::color::PremulRgba8;
use vello_common::filter_effects::{Filter, FilterFunction};
use vello_common::paint::{ImageId, ImageSource, PaintType};
use vello_common::pixmap::Pixmap;
use vello_common::{kurbo, peniko};

use vello_hybrid::{RenderSize, Renderer as VelloRenderer, Resources, Scene, TextureBindings};

use wgpu::CommandEncoder;
use wgpu::{CurrentSurfaceTexture, TextureFormat};

use crate::helpers::brush_to_paint;
use crate::render_command::{BoxShadowCmd, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
use crate::renderer::Renderer;
use crate::sort_commands::SortedCommands;
use crate::text_renderer_data::{TextRenderLine, TextScroll};
use crate::vello_hybrid::render_context::{create_vello_renderer, DeviceHandle, RenderContext, RenderSurface};
use crate::RenderCommand;
use craft_resource_manager::image::ImageResource;
use winit::window::Window;

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

    images: HashMap<ResourceId, (ImageId, Option<DateTime<Utc>>)>,

    resources: Resources,

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
                color: self.surface_clear_color,
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

        let mut images_were_uploaded = false;

        let mut seen_images: HashSet<ImageId> = HashSet::new();
        let mut expired_images: HashSet<ImageId> = HashSet::new();
        let render_list = &self.render_list;
        SortedCommands::draw(&render_list, &render_list.overlay, &mut |command: &RenderCommand| {

            match command {
                RenderCommand::DrawCircle(cmd) => draw_circle(&mut self.scene, cmd),
                RenderCommand::DrawCircleOutline(cmd) => draw_circle_outline(&mut self.scene, cmd),
                RenderCommand::DrawRect(cmd) => draw_rect(&mut self.scene, cmd),
                RenderCommand::DrawRectOutline(cmd) => draw_rect_outline(&mut self.scene, cmd),
                RenderCommand::DrawImage(cmd) => {
                    draw_image(
                        cmd,
                        resource_manager.clone(),
                        &mut expired_images,
                        &mut seen_images,
                        renderer,
                        &mut encoder,
                        device_handle,
                        &mut images_were_uploaded,
                        &mut self.images,
                        &mut self.scene,
                        &mut self.resources
                    );
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

        upload_images(
            &mut expired_images,
            &mut seen_images,
            renderer,
            encoder,
            device_handle,
            &mut images_were_uploaded,
            &mut self.images,
            &mut self.resources
        );
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
            images: HashMap::new(),
            resources: Resources::new(),
            window: window.clone(),
            texture_bindings: TextureBindings::new(),
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
}

fn draw_circle(scene: &mut Scene, cmd: &DrawCircleCmd) {
    scene.set_transform(cmd.transform);
    scene.set_paint(PaintType::from(cmd.color));
    scene.fill_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
}

fn draw_rect(scene: &mut Scene, cmd: &DrawRectCmd) {
    scene.set_transform(cmd.transform);
    scene.set_paint(PaintType::from(cmd.color));
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
    scene.set_paint(PaintType::from(cmd.outline_color));
    scene.stroke_path(&cmd.circle.to_kurbo().to_path(TOLERANCE));
}

fn draw_rect_outline(scene: &mut Scene, cmd: &DrawRectOutlineCmd) {
    scene.set_transform(cmd.transform);
   scene.set_stroke(Stroke::new(cmd.thickness));
   scene.set_paint(PaintType::from(cmd.outline_color));
   scene.stroke_rect(&cmd.rect.to_kurbo());
}

fn draw_image(
    cmd: &DrawImageCmd,
    resource_manager: Arc<ResourceManager>,
    expired_images: &mut HashSet<ImageId>,
    seen_images: &mut HashSet<ImageId>,
    renderer: &mut VelloRenderer,
    encoder: &mut CommandEncoder,
    device_handle: &DeviceHandle,
    images_were_uploaded: &mut bool,
    images: &mut HashMap<ResourceId, (ImageId, Option<DateTime<Utc>>)>,
    scene: &mut Scene,
    resources: &mut Resources
) {
    let resource = resource_manager.get(&cmd.resource_id);

    if let Some(resource) = resource
        && resource.resource_type == "image" && let Some(image) = resource.data.downcast_ref::<ImageResource>()
    {
        //let expiration_time = resource.expiration_time();
        let expiration_time = None;

        // There is an image, and it hasn't expired.
        let image_id = if let Some(stored_image) = images.get(&cmd.resource_id)
            && stored_image.1 == expiration_time
        {
            stored_image.0
        } else {
            // There is an image, but it expired.
            if let Some(stored_image) = images.get(&cmd.resource_id) {
                expired_images.insert(stored_image.0);
            }

            *images_were_uploaded = true;

            // NOTE: We may be able to avoid this if we implement the AtlasWriter trait.
            let premul_data: Vec<PremulRgba8> = image
                .image
                .to_vec()
                .chunks_exact(4)
                .map(|rgba| {
                    let alpha = u16::from(rgba[3]);
                    let premultiply = |component| (alpha * (u16::from(component)) / 255) as u8;
                    PremulRgba8 {
                        r: premultiply(rgba[0]),
                        g: premultiply(rgba[1]),
                        b: premultiply(rgba[2]),
                        a: alpha as u8,
                    }
                })
                .collect();
            let pixmap = Pixmap::from_parts(premul_data, image.image.width() as u16, image.image.height() as u16);

            let image_id = renderer.upload_image(
                resources,
                &device_handle.device,
                &device_handle.queue,
                encoder,
                &pixmap,
            );
            images.insert(cmd.resource_id.clone(), (image_id, expiration_time));

            image_id
        };
        seen_images.insert(image_id);

        let mut transform = Affine::IDENTITY;
        transform = transform.with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
        transform = transform.pre_scale_non_uniform(
            cmd.rect.width as f64 / image.image.width() as f64,
            cmd.rect.height as f64 / image.image.height() as f64,
        );
        scene.set_transform(cmd.transform * transform);

        scene.set_paint(PaintType::Image(vello_common::paint::Image {
            image: ImageSource::OpaqueId {
                id: image_id,
                may_have_transparency: true,
            },
            sampler: Default::default(),
        }));

        scene.fill_rect(&kurbo::Rect::new(0.0, 0.0, image.image.width() as f64, image.image.height() as f64));
    }
}

fn upload_images(
    expired_images: &mut HashSet<ImageId>,
    seen_images: &mut HashSet<ImageId>,
    renderer: &mut VelloRenderer,
    mut encoder: CommandEncoder,
    device_handle: &DeviceHandle,
    images_were_uploaded: &mut bool,
    images: &mut HashMap<ResourceId, (ImageId, Option<DateTime<Utc>>)>,
    resources: &mut Resources
) {
    let mut to_remove: Vec<ResourceId> = Vec::new();
    for expired_image_id in expired_images.iter() {
        // Note: Expired images will have an entry in the images hashmap, but with a new ImageId.
        // Meaning, that we need to delete the detached/abandoned expired image id here.
        renderer.destroy_image(
            resources,
            &device_handle.device,
            &device_handle.queue,
            &mut encoder,
            *expired_image_id,
        );
        *images_were_uploaded = true;
    }

    for (key, (image_id, _)) in images.iter() {
        let seen = seen_images.contains(image_id);
        let expired = expired_images.contains(image_id);

        // Delete the culled image, only if it hasn't already been deleted when we looped over the expired images.
        if !seen && !expired {
            renderer.destroy_image(
                resources,
                &device_handle.device,
                &device_handle.queue,
                &mut encoder,
                *image_id,
            );
            *images_were_uploaded = true;
        }

        if !seen {
            to_remove.push(key.clone());
        }
    }
    for key in &to_remove {
        images.remove(key);
    }

    // Submit the texture write commands.
    if *images_were_uploaded {
        device_handle.queue.submit([encoder.finish()]);
    }
}

fn draw_text(cmd: &DrawTextCmd, scene: &mut Scene, resources: &mut Resources, window: &Rectangle) {
    let text_transform =
        Affine::default().with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
    let scroll = cmd.text_scroll.unwrap_or(TextScroll::default()).scroll_y;
    let text_transform = text_transform.then_translate(kurbo::Vec2::new(0.0, -scroll as f64));
    let transformed_container = Rectangle::from_kurbo(cmd.transform.transform_rect_bbox(cmd.rect.to_kurbo()));

    let c = cmd.data.upgrade();
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
                    let gy = first_glyph.y + transformed_container.y - scroll;
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
                x: background.x + cmd.rect.x,
                y: -scroll + background.y + cmd.rect.y,
                width: background.width,
                height: background.height,
            };
            draw_rect(scene, &DrawRectCmd {
                rect: background_rect,
                color: *color,
                transform: cmd.transform
            });
        }

        for (selection, selection_color) in &line.selections {
            let selection_rect = Rectangle {
                x: selection.x + cmd.rect.x,
                y: -scroll + selection.y + cmd.rect.y,
                width: selection.width,
                height: selection.height,
            };
            draw_rect(scene, &DrawRectCmd {
                rect: selection_rect,
                color: *selection_color,
                transform: cmd.transform
            });
        }
    });

    scene.set_transform(cmd.transform * text_transform);

    cull_and_process(&mut |line: &TextRenderLine| {
        for item in &line.items {
            if let Some(underline) = &item.underline {
                scene.set_stroke(Stroke::new(underline.width.into()));
                scene.set_paint(PaintType::from(underline.brush.color));
                scene.stroke_path(&underline.line.to_path(0.1));
            }

            scene.set_paint(PaintType::from(
                text_render
                    .override_brush
                    .map(|b| b.color)
                    .unwrap_or_else(|| item.brush.color),
            ));

            let glyph_run_builder = scene
                .glyph_run(resources, &item.font)
                //.atlas_cache(true)
                .font_size(item.font_size);
            glyph_run_builder.fill_glyphs(item.glyphs.iter().map(|glyph| Glyph {
                id: glyph.id,
                x: glyph.x,
                y: glyph.y,
            }));
        }
    });

    if cmd.show_cursor
        && let Some((cursor, cursor_color)) = &text_render.cursor
    {
        let cursor_rect = Rectangle {
            x: cursor.x + cmd.rect.x,
            y: -scroll + cursor.y + cmd.rect.y,
            width: cursor.width,
            height: cursor.height,
        };
        draw_rect(scene, &DrawRectCmd {
            rect: cursor_rect,
            color: *cursor_color,
            transform: cmd.transform
        });
    }
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
