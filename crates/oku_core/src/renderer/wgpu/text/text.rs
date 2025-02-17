use crate::components::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::geometry::Rectangle;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::text::caching::{ContentType, GlyphInfo, TextAtlas};
use crate::renderer::wgpu::text::pipeline::{TextPipeline, TextPipelineConfig, DEFAULT_TEXT_PIPELINE_CONFIG};
use crate::renderer::wgpu::text::vertex::TextVertex;
use crate::renderer::wgpu::PerFrameData;
use cosmic_text::fontdb::ID;
use cosmic_text::{Buffer, Cursor, Edit, Editor, FontSystem, LayoutGlyph, LayoutRun, SwashCache};
use peniko::kurbo::{Point, Rect, Size};
use std::cmp;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use wgpu::util::DeviceExt;
use wgpu::RenderPass;

pub struct TextRenderInfo {
    pub(crate) element_id: ComponentId,
    pub(crate) rectangle: Rectangle,
    pub(crate) fill_color: Color,
}

pub(crate) struct TextRenderer {
    pub(crate) cached_pipelines: HashMap<TextPipelineConfig, TextPipeline>,
    pub(crate) text_areas: Vec<TextRenderInfo>,
    pub(crate) swash_cache: SwashCache,
    pub(crate) text_atlas: TextAtlas,
    pub(crate) vertices: Vec<TextVertex>,
    pub(crate) indices: Vec<u32>,
}

impl TextRenderer {
    pub(crate) fn new(context: &Context) -> Self {
        let max_texture_size = context.device.limits().max_texture_dimension_2d;
        let mut renderer = TextRenderer {
            cached_pipelines: Default::default(),
            text_areas: vec![],
            swash_cache: SwashCache::new(),
            text_atlas: TextAtlas::new(&context.device, max_texture_size, max_texture_size),
            vertices: vec![],
            indices: vec![],
        };

        renderer.cached_pipelines.insert(
            DEFAULT_TEXT_PIPELINE_CONFIG,
            TextPipeline::new_pipeline_with_configuration(context, DEFAULT_TEXT_PIPELINE_CONFIG)
        );

        renderer
    }

    pub(crate) fn build(&mut self, rectangle: Rectangle, component_id: ComponentId, color: Color) {
        self.text_areas.push(TextRenderInfo {
            element_id: component_id,
            rectangle,
            fill_color: color,
        });
    }

    pub(crate) fn prepare(&mut self, context: &Context, font_system: &mut FontSystem, element_state: &ElementStateStore) -> Option<PerFrameData> {

        for text_area in self.text_areas.iter() { 
            if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().data.downcast_ref::<TextInputState>() {
                
                let editor = &text_context.editor;
                let buffer_glyphs = create_glyphs_for_editor(
                    editor,
                    text_area.fill_color,
                    Color::from_rgb8(0, 0, 0),
                    Color::from_rgb8(0, 120, 215),
                    Color::from_rgb8(255, 255, 255),
                );

                // Draw the Glyphs
                for buffer_line in &buffer_glyphs.buffer_lines {
                    
                    // Draw the highlights
                    for glyph_highlight in &buffer_line.glyph_highlights {

                        let width = glyph_highlight.width() as f32;
                        let height = glyph_highlight.height() as f32;
                        
                        build_rectangle(ContentType::Rectangle, Rectangle {
                            x: text_area.rectangle.x + glyph_highlight.x0 as f32,
                            y: text_area.rectangle.y + glyph_highlight.y0 as f32,
                            width,
                            height,
                        }, buffer_glyphs.glyph_highlight_color, &mut self.vertices, &mut self.indices);
                        
                    }

                    // Draw the cursor
                    if let Some(cursor) = &buffer_line.cursor {
                        build_rectangle(ContentType::Rectangle, Rectangle {
                            x: text_area.rectangle.x + cursor.x0 as f32,
                            y: text_area.rectangle.y + cursor.y0 as f32,
                            width: cursor.width() as f32,
                            height: cursor.height() as f32,
                        }, buffer_glyphs.cursor_color, &mut self.vertices, &mut self.indices);
                    }

                    // Draw the glyphs
                    for glyph_run in &buffer_line.glyph_runs {
                        let glyph_color = glyph_run.glyph_color;

                        for glyph in glyph_run.glyphs.iter() {
                            let physical_glyph = glyph.physical((0., 0.), 1.0);

                            // Check if the image is available in the cache
                            let glyph_info: Option<GlyphInfo> = if let Some(glyph_info) = self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key) {
                                Some(glyph_info)
                            } else if let Some(image) = self.swash_cache.get_image(font_system, physical_glyph.cache_key) {
                                self.text_atlas.add_glyph(image, physical_glyph.cache_key, &context.queue);

                                self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key)
                            } else {
                                None
                            };

                            if let Some(glyph_info) = glyph_info {
                                let rel_gylh_x = physical_glyph.x + glyph_info.swash_image_placement.left;
                                let rel_gylh_y = glyph_run.line_y as i32 + physical_glyph.y + (-glyph_info.swash_image_placement.top);
                                build_glyph_rectangle(self.text_atlas.texture_width, self.text_atlas.texture_height, glyph_info.clone(), Rectangle {
                                    x: text_area.rectangle.x + rel_gylh_x as f32,
                                    y: text_area.rectangle.y + rel_gylh_y as f32,
                                    width: glyph_info.width as f32,
                                    height: glyph_info.height as f32,
                                }, glyph_color, &mut self.vertices, &mut self.indices);
                            }

                        }
                    }
                }

            } else if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().data.downcast_ref::<TextState>() {
                for run in text_context.buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical_glyph = glyph.physical((0., 0.), 1.0);

                        let glyph_color = match glyph.color_opt {
                            Some(some) => Color::from_rgba8(some.r(), some.g(), some.b(), some.a()),
                            None => text_area.fill_color,
                        };

                        // Check if the image is available in the cache
                        let glyph_info: Option<GlyphInfo> = if let Some(glyph_info) = self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key) {
                            Some(glyph_info)
                        } else if let Some(image) = self.swash_cache.get_image(font_system, physical_glyph.cache_key) {
                            self.text_atlas.add_glyph(image, physical_glyph.cache_key, &context.queue);

                            self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key)
                        } else {
                            None
                        };
                        
                        if let Some(glyph_info) = glyph_info {
                            let rel_gylh_x = physical_glyph.x + glyph_info.swash_image_placement.left;
                            let rel_gylh_y = run.line_y as i32 + physical_glyph.y + (-glyph_info.swash_image_placement.top);
                            build_glyph_rectangle(self.text_atlas.texture_width, self.text_atlas.texture_height, glyph_info.clone(), Rectangle {
                                x: text_area.rectangle.x + rel_gylh_x as f32,
                                y: text_area.rectangle.y + rel_gylh_y as f32,
                                width: glyph_info.width as f32,
                                height: glyph_info.height as f32,
                            }, glyph_color, &mut self.vertices, &mut self.indices);   
                        }

                    }
                }

                
            } else {
                panic!("Unknown state provided to the renderer!");
            }
        }
        
        if self.indices.is_empty() {
            self.text_areas.clear();
            return None;
        }

        let indices = self.indices.len();
        let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.vertices.clear();
        self.indices.clear();
        self.text_areas.clear();

        Some(PerFrameData {
            vertex_buffer,
            index_buffer,
            indices
        })
    }
    
    pub(crate) fn draw(&mut self, context: &mut Context, render_pass: &mut RenderPass, per_frame_data: &PerFrameData) {
        let text_pipeline = self.cached_pipelines.get(&DEFAULT_TEXT_PIPELINE_CONFIG).unwrap();

        let texture_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let texture_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.text_atlas.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.text_atlas.texture_sampler),
                },
            ],
            label: Some("oku_bind_group"),
        });
        
        render_pass.set_pipeline(&text_pipeline.pipeline);
        render_pass.set_bind_group(0, Some(&texture_bind_group), &[]);
        render_pass.set_bind_group(1, Some(&context.global_buffer.bind_group), &[]);
        render_pass.set_vertex_buffer(0, per_frame_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(per_frame_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(per_frame_data.indices as u32), 0, 0..1);
    }
}

pub(crate) fn build_rectangle(content_type: ContentType,
                              rectangle: Rectangle,
                              fill_color: Color,
                              vertices: &mut Vec<TextVertex>,
                              indices: &mut Vec<u32>) {
    let x = rectangle.x;
    let y = rectangle.y;
    let width = rectangle.width;
    let height = rectangle.height;

    let top_left = glam::vec4(x, y, 0.0, 1.0);
    let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
    let top_right = glam::vec4(x + width, y, 0.0, 1.0);
    let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);
    
    let content_type = content_type as u32;
    vertices.append(&mut vec![
        TextVertex {
            position: [top_left.x, top_left.y, top_left.z],
            uv: [0.0, 0.0],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [bottom_left.x, bottom_left.y, bottom_left.z],
            uv: [0.0, 0.0],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [top_right.x, top_right.y, top_right.z],
            uv: [0.0, 0.0],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [bottom_right.x, bottom_right.y, bottom_right.z],
            uv: [0.0, 0.0],
            background_color: fill_color.components,
            content_type
        },
    ]);

    let next_starting_index: u32 = (indices.len() / 6) as u32 * 4;
    indices.append(&mut vec![
        next_starting_index,
        next_starting_index + 1,
        next_starting_index + 2,
        next_starting_index + 2,
        next_starting_index + 1,
        next_starting_index + 3,
    ]);
}

pub(crate) fn build_glyph_rectangle(
                                    text_atlas_texture_width: u32,
                                    text_atlas_texture_height: u32,
                                    glyph_info: GlyphInfo,
                                    rectangle: Rectangle,
                                    fill_color: Color,
                                    vertices: &mut Vec<TextVertex>,
                                    indices: &mut Vec<u32>) {
    let x = rectangle.x;
    let y = rectangle.y;
    let width = rectangle.width;
    let height = rectangle.height;

    let top_left = glam::vec4(x, y, 0.0, 1.0);
    let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
    let top_right = glam::vec4(x + width, y, 0.0, 1.0);
    let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

    let left_text_corod = glyph_info.texture_coordinate_x as f32 / text_atlas_texture_width as f32;
    let top_tex_coord = glyph_info.texture_coordinate_y as f32 / text_atlas_texture_height as f32;
    
    let content_type = glyph_info.content_type as u32;
    vertices.append(&mut vec![
        TextVertex {
            position: [top_left.x, top_left.y, top_left.z],
            uv: [left_text_corod, top_tex_coord],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [bottom_left.x, bottom_left.y, bottom_left.z],
            uv: [left_text_corod, top_tex_coord + (rectangle.height / text_atlas_texture_height as f32)],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [top_right.x, top_right.y, top_right.z],
            uv: [left_text_corod + (rectangle.width / text_atlas_texture_width as f32), top_tex_coord],
            background_color: fill_color.components,
            content_type
        },

        TextVertex {
            position: [bottom_right.x, bottom_right.y, bottom_right.z],
            uv: [left_text_corod + (rectangle.width / text_atlas_texture_width as f32), top_tex_coord + (rectangle.height / text_atlas_texture_height as f32)],
            background_color: fill_color.components,
            content_type
        },
    ]);

    let next_starting_index: u32 = (indices.len() / 6) as u32 * 4;
    indices.append(&mut vec![
        next_starting_index,
        next_starting_index + 1,
        next_starting_index + 2,
        next_starting_index + 2,
        next_starting_index + 1,
        next_starting_index + 3,
    ]);
}

pub(crate) struct EditorInfo {
    cursor_color: Color,
    selection_color: Color,
    selected_text_color: Color,
    selection_bounds: Option<(Cursor, Cursor)>,
    cursor: Cursor,
}

impl EditorInfo {
    fn new(
        editor: &Editor,
        cursor_color: Color,
        selection_color: Color,
        selected_text_color: Color,
    ) -> Self {
        Self {
            cursor_color,
            selection_color,
            selected_text_color,
            selection_bounds: editor.selection_bounds(),
            cursor: editor.cursor(),
        }
    }
}


pub(crate) fn create_glyphs_for_editor(
    editor: &Editor,
    text_color: Color,
    cursor_color: Color,
    selection_color: Color,
    selected_text_color: Color,
) -> BufferGlyphs {
    editor.with_buffer(|buffer| {
        create_glyphs(
            buffer,
            text_color,
            Some(EditorInfo::new(
                editor,
                cursor_color,
                selection_color,
                selected_text_color,
            )),
        )
    })
}

pub(crate) fn create_glyphs(
    buffer: &Buffer,
    text_color: Color,
    editor_info: Option<EditorInfo>,
) -> BufferGlyphs {
    // Get the laid out glyphs and convert them to Glyphs for vello

    let mut last_font: Option<(ID, Color)> = None;

    let mut buffer_glyphs = BufferGlyphs {
        font_size: buffer.metrics().font_size,
        glyph_highlight_color: Color::WHITE,
        cursor_color: Color::BLACK,
        buffer_lines: vec![],
    };

    if let Some(editor_info) = &editor_info {
        buffer_glyphs.cursor_color = editor_info.cursor_color;
        buffer_glyphs.glyph_highlight_color = editor_info.selection_color;
    }

    for layout_run in buffer.layout_runs() {
        let mut current_glyphs: Vec<LayoutGlyph> = vec![];
        let line_i = layout_run.line_i;
        let line_y = layout_run.line_y as f64;
        let line_top = layout_run.line_top as f64;
        let line_height = layout_run.line_height as f64;

        let mut buffer_line = BufferLine {
            glyph_highlights: vec![],
            cursor: None,
            glyph_runs: vec![],
        };

        if let Some(editor_info) = &editor_info {
            // Highlight selection
            if let Some((start, end)) = editor_info.selection_bounds {
                if line_i >= start.line && line_i <= end.line {
                    let mut range_opt = None;
                    for glyph in layout_run.glyphs.iter() {
                        // Guess x offset based on characters
                        let cluster = &layout_run.text[glyph.start..glyph.end];
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
                                buffer_line.glyph_highlights.push(Rect::from_origin_size(
                                    Point::new(min as f64, line_top),
                                    Size::new(cmp::max(0, max - min) as f64, line_height),
                                ));
                            }
                            c_x += c_w;
                        }
                    }

                    if layout_run.glyphs.is_empty() && end.line > line_i {
                        // Highlight all internal empty lines
                        range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                    }

                    if let Some((mut min, mut max)) = range_opt.take() {
                        if end.line > line_i {
                            // Draw to end of line
                            if layout_run.rtl {
                                min = 0;
                            } else {
                                max = buffer.size().0.unwrap_or(0.0) as i32;
                            }
                        }
                        buffer_line.glyph_highlights.push(Rect::from_origin_size(
                            Point::new(min as f64, line_top),
                            Size::new(cmp::max(0, max - min) as f64, line_height),
                        ));
                    }
                }
            }

            // Cursor
            if let Some((x, y)) = cursor_position(&editor_info.cursor, &layout_run) {
                buffer_line.cursor = Some(Rect::from_origin_size(
                    Point::new(x as f64, y as f64),
                    Size::new(1.0, line_height),
                ));
            }
        }

        for glyph in layout_run.glyphs {
            let mut glyph_color = match glyph.color_opt {
                Some(color) => Color::from_rgba8(color.r(), color.g(), color.b(), color.a()),
                None => text_color,
            };

            if let Some(editor_info) = &editor_info {
                if text_color != editor_info.selected_text_color {
                    if let Some((start, end)) = editor_info.selection_bounds {
                        if line_i >= start.line
                            && line_i <= end.line
                            && (start.line != line_i || glyph.end > start.index)
                            && (end.line != line_i || glyph.start < end.index)
                        {
                            glyph_color = editor_info.selected_text_color;
                        }
                    }
                }
            }

            if let Some((last_font, last_glyph_color)) = last_font {
                if last_font != glyph.font_id || last_glyph_color != glyph_color {
                    buffer_line.glyph_runs.push(BufferGlyphRun {
                        font: last_font,
                        glyphs: current_glyphs,
                        glyph_color: last_glyph_color,
                        line_y,
                    });
                    current_glyphs = vec![];
                }
            }

            last_font = Some((glyph.font_id, glyph_color));
            current_glyphs.push(glyph.clone());
        }
        if !current_glyphs.is_empty() {
            let (last_font, last_color) = last_font.unwrap();
            buffer_line.glyph_runs.push(BufferGlyphRun {
                font: last_font,
                glyphs: current_glyphs,
                glyph_color: last_color,
                line_y,
            });
        }

        buffer_glyphs.buffer_lines.push(buffer_line);
    }

    buffer_glyphs
}

// Copied directly from cosmic_text.
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

// Copied directly from cosmic_text.
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

pub(crate) struct BufferGlyphs {
    pub(crate) font_size: f32,
    pub(crate) glyph_highlight_color: Color,
    pub(crate) cursor_color: Color,
    pub(crate) buffer_lines: Vec<BufferLine>,
}

pub(crate) struct BufferLine {
    pub(crate) glyph_highlights: Vec<Rect>,
    pub(crate) cursor: Option<Rect>,
    pub(crate) glyph_runs: Vec<BufferGlyphRun>,
}

pub(crate) struct BufferGlyphRun {
    pub(crate) font: ID,
    pub(crate) glyphs: Vec<LayoutGlyph>,
    pub(crate) glyph_color: Color,
    pub(crate) line_y: f64,
}