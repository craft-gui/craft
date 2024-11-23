use std::cmp;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use vello::{kurbo, peniko, AaConfig, Glyph, RendererOptions};
use crate::components::component::ComponentId;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand, Renderer};
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::state_store::StateStore;
use cosmic_text::{Cursor, Edit, Editor, FontSystem, LayoutRun, SwashCache};
use std::sync::Arc;
use cosmic_text::fontdb::{Query, ID};
use tiny_skia::{ColorSpace, Paint, PixmapPaint, PixmapRef, Transform};
use tokio::sync::RwLockReadGuard;
use unicode_segmentation::UnicodeSegmentation;
use vello::kurbo::{Affine, Circle, Ellipse, Line, Rect, RoundedRect, Stroke, Vec2};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::Scene;
use vello::util::{RenderContext, RenderSurface};
use winit::window::Window;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::platform::resource_manager::resource::Resource;

pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<dyn Window>,
}

enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<dyn Window>>),
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
    cache: SwashCache,
    vello_fonts: HashMap<cosmic_text::fontdb::ID, peniko::Font>,
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> vello::Renderer {
    vello::Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport {
                area: false,
                msaa8: false,
                msaa16: true,
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
            state: RenderState::Suspended(None),
            scene: Scene::new(),
            cache: SwashCache::new(),
            surface_clear_color: Color::rgba(255, 255, 255, 255),
            vello_fonts: HashMap::new(),
        };

        // Create a vello Surface
        let surface_size = window.surface_size();

        let surface = vello_renderer.context.create_surface(
            window.clone(),
            surface_size.width,
            surface_size.height,
            wgpu::PresentMode::AutoVsync,
        ).await.unwrap();

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState { window, surface });

        vello_renderer
    }
}

fn to_vello_rgba_f32_color(color: Color) -> vello::peniko::Color {
    vello::peniko::Color::rgba(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0, color.a as f64 / 255.0)
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {

    let rect = Rect::new(rectangle.x as f64, rectangle.y as f64,
                         (rectangle.x + rectangle.width) as f64,
                         (rectangle.y + rectangle.height) as f64
    );
    scene.fill(Fill::NonZero, Affine::IDENTITY, to_vello_rgba_f32_color(fill_color), None, &rect);
}

fn draw_editor(scene: &mut Scene, editor: &Editor, vello_fonts: &HashMap<ID, peniko::Font>, offset: Rectangle, text_color: Color) {

    let selection_color = Color::rgba(0, 0, 255, 255);
    let selected_text_color = Color::rgba(255, 255, 255, 255);
    let cursor_color = Color::rgba(0, 0, 0, 255);

    let mut buffer_glyphs = BufferGlyphs {
        font_size: editor.with_buffer(|buffer| buffer.metrics().font_size),
        glyphs: vec![],
    };

    let selection_bounds = editor.selection_bounds();
    editor.with_buffer(|buffer| {
        let mut last_font: Option<(ID, Color)> = None;


        let mut current_glyphs: Vec<Glyph> = vec![];

        for run in buffer.layout_runs() {
            let line_i = run.line_i;
            let line_y = run.line_y as f64;
            let line_top = run.line_top;
            let line_height = run.line_height;

            // Highlight selection
            if let Some((start, end)) = selection_bounds {
                if line_i >= start.line && line_i <= end.line {
                    let mut range_opt = None;
                    for glyph in run.glyphs.iter() {
                        // Guess x offset based on characters
                        let cluster = &run.text[glyph.start..glyph.end];
                        let total = cluster.grapheme_indices(true).count();
                        let mut c_x = glyph.x;
                        let c_w = glyph.w / total as f32;
                        for (i, c) in cluster.grapheme_indices(true) {
                            let c_start = glyph.start + i;
                            let c_end = glyph.start + i + c.len();
                            if (start.line != line_i || c_end > start.index)
                                && (end.line != line_i || c_start < end.index)
                            {
                                range_opt = match range_opt.take() {
                                    Some((min, max)) => Some((
                                        cmp::min(min, c_x as i32),
                                        cmp::max(max, (c_x + c_w) as i32),
                                    )),
                                    None => Some((c_x as i32, (c_x + c_w) as i32)),
                                };
                            } else if let Some((min, max)) = range_opt.take() {
                                vello_draw_rect(
                                    scene,
                                    Rectangle {
                                        x: min as f32 + offset.x,
                                        y: line_top as i32 as f32 + offset.y,
                                        width: cmp::max(0, max - min) as f32,
                                        height: line_height,
                                    },
                                    selection_color
                                );
                            }
                            c_x += c_w;
                        }
                    }

                    if run.glyphs.is_empty() && end.line > line_i {
                        // Highlight all of internal empty lines
                        range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                    }

                    if let Some((mut min, mut max)) = range_opt.take() {
                        if end.line > line_i {
                            // Draw to end of line
                            if run.rtl {
                                min = 0;
                            } else {
                                max = buffer.size().0.unwrap_or(0.0) as i32;
                            }
                        }
                        vello_draw_rect(
                            scene,
                            Rectangle {
                                x: min as f32 + offset.x,
                                y: line_top as i32 as f32 + offset.y,
                                width: cmp::max(0, max - min) as f32,
                                height: line_height,
                            },
                            selection_color
                        );
                    }
                }
            }

            // Draw cursor
            if let Some((x, y)) = cursor_position(&editor.cursor(), &run) {
                vello_draw_rect(scene, Rectangle::new(x as f32 + offset.x, y as f32 + offset.y, 1.0, line_height), cursor_color);
            }

            for glyph in run.glyphs.iter() {
                let mut glyph_color = match glyph.color_opt {
                    Some(some) => {
                        let color = some.as_rgba();
                        Color::rgba(color[0], color[1], color[2], color[3])
                    },
                    None => text_color,
                };
                if text_color != selected_text_color {
                    if let Some((start, end)) = selection_bounds {
                        if line_i >= start.line
                            && line_i <= end.line
                            && (start.line != line_i || glyph.end > start.index)
                            && (end.line != line_i || glyph.start < end.index)
                        {
                            glyph_color = selected_text_color;
                        }
                    }
                }

                if let Some((last_font, last_color)) = last_font {
                    if last_font != glyph.font_id || last_color != glyph_color {
                        buffer_glyphs.glyphs.push(BufferGlyphRun {
                            font: last_font,
                            glyphs: current_glyphs,
                            line_y,
                            color: last_color,
                        });
                        current_glyphs = vec![];
                    }
                }
                last_font = Some((glyph.font_id, glyph_color));
                current_glyphs.push(Glyph {
                    x: glyph.x,
                    y: glyph.y,
                    id: glyph.glyph_id as u32,
                });
            }
            if !current_glyphs.is_empty() {
                let (last_font, last_color) = last_font.unwrap();
                buffer_glyphs.glyphs.push(BufferGlyphRun {
                    font: last_font,
                    glyphs: current_glyphs,
                    line_y,
                    color: last_color,
                });
                current_glyphs = vec![];
            }
        }
    });

    let text_transform = Affine::translate((offset.x as f64, offset.y as f64));

    // Draw the Glyphs
    for glyph_run in buffer_glyphs.glyphs.iter() {
        let font = vello_fonts.get(&glyph_run.font).unwrap();
        let glyphs = glyph_run.glyphs.clone();
        let color = glyph_run.color;
        scene
            .draw_glyphs(font)
            .font_size(buffer_glyphs.font_size)
            .brush(to_vello_rgba_f32_color(color))
            .transform(text_transform.then_translate(Vec2::new(0.0, glyph_run.line_y)))
            .draw(vello::peniko::Fill::NonZero, glyphs.into_iter());
    }

}

struct BufferGlyphs {
    font_size: f32,
    glyphs: Vec<BufferGlyphRun>,
}

struct BufferGlyphRun {
    font: ID,
    glyphs: Vec<Glyph>,
    line_y: f64,
    color: Color,
}

fn cursor_glyph_opt(cursor: &Cursor, run: &LayoutRun) -> Option<(usize, f32)> {
    if cursor.line == run.line_i {
        for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
            if cursor.index == glyph.start {
                return Some((glyph_i, 0.0));
            } else if cursor.index > glyph.start && cursor.index < glyph.end {
                // Guess x offset based on characters
                let mut before = 0;
                let mut total = 0;

                let cluster = &run.text[glyph.start..glyph.end];
                for (i, _) in cluster.grapheme_indices(true) {
                    if glyph.start + i < cursor.index {
                        before += 1;
                    }
                    total += 1;
                }

                let offset = glyph.w * (before as f32) / (total as f32);
                return Some((glyph_i, offset));
            }
        }
        match run.glyphs.last() {
            Some(glyph) => {
                if cursor.index == glyph.end {
                    return Some((run.glyphs.len(), 0.0));
                }
            }
            None => {
                return Some((0, 0.0));
            }
        }
    }
    None
}

fn cursor_position(cursor: &Cursor, run: &LayoutRun) -> Option<(i32, i32)> {
    let (cursor_glyph, cursor_glyph_offset) = cursor_glyph_opt(cursor, run)?;
    let x = match run.glyphs.get(cursor_glyph) {
        Some(glyph) => {
            // Start of detected glyph
            if glyph.level.is_rtl() {
                (glyph.x + glyph.w - cursor_glyph_offset) as i32
            } else {
                (glyph.x + cursor_glyph_offset) as i32
            }
        }
        None => match run.glyphs.last() {
            Some(glyph) => {
                // End of last glyph
                if glyph.level.is_rtl() {
                    glyph.x as i32
                } else {
                    (glyph.x + glyph.w) as i32
                }
            }
            None => {
                // Start of empty line
                0
            }
        },
    };

    Some((x, run.line_top as i32))
}

impl Renderer for VelloRenderer<'_> {
    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => {
                active_render_state.window.surface_size().width as f32
            }
            RenderState::Suspended(_) => {
                0.0
            }
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => {
                active_render_state.window.surface_size().height as f32
            }
            RenderState::Suspended(_) => {
                0.0
            }
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

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {}

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

    fn load_font(&mut self, font_system: &mut FontSystem) {
        let font_faces: Vec<(cosmic_text::fontdb::ID, u32)> = font_system.db().faces().map(|face| (face.id, face.index)).collect();
        for (font_id, index) in font_faces {
            let font = font_system.get_font(font_id).unwrap();
            let resource = Arc::new(font.data().to_vec());
            let blob = Blob::new(resource);
            let vello_font = peniko::Font::new(blob, index);
            self.vello_fonts.insert(font_id, vello_font);
        }
        
    }
    

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &StateStore,
    ) {
        self.scene.reset();

        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };
        
        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(&mut self.scene, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    // vello_draw_rect_outline(&mut self.scene, rectangle, outline_color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        let data = Arc::new(image.clone().into_raw().to_vec());
                        let blob = Blob::new(data);
                        let vello_image = peniko::Image::new(blob, peniko::Format::Rgba8, image.width() as u32, image.height() as u32);

                        let mut transform= Affine::IDENTITY;
                        transform = transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            rectangle.width as f64 / image.width() as f64,
                            rectangle.height as f64 / image.height() as f64,
                        );

                        self.scene.draw_image(&vello_image, transform);

                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color) => {
                    let clip = Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);

                    self.scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                    if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().downcast_ref::<TextInputState>()
                    {
                        let editor = &text_context.editor;
                        draw_editor(&mut self.scene, editor, &self.vello_fonts, rect, fill_color);
                    } else if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().downcast_ref::<TextState>()
                    {
                        let buffer = &text_context.buffer;

                        for layout_run in buffer.layout_runs() {
                            let font_id = layout_run.glyphs[0].font_id;
                            let glyphs: Vec<Glyph> = layout_run.glyphs.iter().map(|glyph| {
                                let x = glyph.x;
                                let y = glyph.y;

                                Glyph {
                                    id: glyph.glyph_id as u32,
                                    x,
                                    y,
                                }
                            }).collect();

                            let transform = vello::kurbo::Affine::translate((
                                rect.x as f64,
                                rect.y as f64 + layout_run.line_y as f64,
                            ));

                            let key: Vec<&cosmic_text::fontdb::ID> = self.vello_fonts.keys().collect();
                            self.scene.draw_glyphs(&self.vello_fonts[&font_id])
                                .font_size(buffer.metrics().font_size)
                                .brush(vello::peniko::Color::rgba(
                                    0.0,
                                    0.0,
                                    0.0,
                                    1.0,
                                ))
                                .transform(transform)
                                .draw(
                                peniko::Fill::NonZero,
                                glyphs.into_iter()
                            );
                        }
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };
                    self.scene.pop_layer();
                },
                /*RenderCommand::PushTransform(transform) => {
                    self.scene.push_transform(transform);
                },
                RenderCommand::PopTransform => {
                    self.scene.pop_transform();
                },*/
                RenderCommand::PushLayer(rect) => {
                    let clip = Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);
                    self.scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                },
                RenderCommand::PopLayer => {
                    self.scene.pop_layer();
                },
            }
        }
        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

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
                    base_color: to_vello_rgba_f32_color(self.surface_clear_color),
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();

    }
}
