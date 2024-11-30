use cosmic_text::{Buffer, BufferRef, CacheKey, Edit, FontSystem, SwashCache};
use cosmic_text::fontdb::ID;
use image::{ColorType, DynamicImage, GenericImage, GrayImage, Luma};
use tokio::sync::RwLockReadGuard;
use wgpu::{MultisampleState, RenderPass};
use crate::components::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand};
use crate::engine::renderer::wgpu::context::Context;
use crate::engine::renderer::wgpu::pipeline_2d::TextRenderInfo;
use crate::engine::renderer::wgpu::render_group::ClipRectangle;
use crate::platform::resource_manager::ResourceManager;
use crate::reactive::state_store::StateStore;

pub(crate) struct TextRenderer {
    pub(crate) text_areas: Vec<TextRenderInfo>,
    pub(crate) swash_cache: SwashCache,
    x_offset: u32,
    y_offset: u32,
    text_atlas: DynamicImage,
}

impl TextRenderer {
    pub(crate) fn new(context: &Context) -> Self {

        let mut text_atlas = DynamicImage::new(1024, 1024, image::ColorType::Rgba8);
        
        Self {
            text_atlas,
            text_areas: vec![],
            swash_cache: SwashCache::new(),
            x_offset: 0,
            y_offset: 0,
        }
    }

    pub(crate) fn build(&mut self, rectangle: Rectangle, component_id: ComponentId, color: Color) {
        self.text_areas.push(TextRenderInfo {
            element_id: component_id,
            rectangle,
            fill_color: color,
        });
    }

    pub(crate) fn prepare(&mut self, context: &Context, font_system: &mut FontSystem, element_state: &StateStore, clip_rectangle: ClipRectangle) {

        for text_area in self.text_areas.iter() { 
            if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextInputState>() {
                let text_buffer = match text_context.editor.buffer_ref() {
                    BufferRef::Owned(buffer) => buffer,
                    BufferRef::Borrowed(_) => panic!("Editor must own buffer."),
                    BufferRef::Arc(_) => panic!("Editor must own buffer."),
                };
            } else if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextState>() {
                let buffer_glyphs = create_glyphs(&text_context.buffer, Color::BLACK);
                
                let row_height = 50; // You can adjust this to fit your glyph height

                for run in text_context.buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical_glyph = glyph.physical((0., 0.), 1.0);

                        let glyph_color = match glyph.color_opt {
                            Some(some) => some,
                            None => cosmic_text::Color::rgba(0, 0, 0, 255),
                        };

                        // Check if the image is available in the cache
                        
                        
                        
                        if let Some(image) = self.swash_cache.get_image(font_system, physical_glyph.cache_key) {
                            if image.placement.height == 0 {
                                continue;
                            }
                            
                            let glyph_width = image.placement.width as u32;
                            let glyph_height = image.placement.height as u32;

                            // Check if the glyph fits in the current row
                            if self.x_offset + glyph_width > self.text_atlas.width() {
                                // Move to the next row
                                self.x_offset = 0;
                                self.y_offset += row_height; // Adjust as necessary based on your glyph heights
                            }

                            // Ensure we don't exceed the atlas height
                            if self.y_offset + glyph_height > self.text_atlas.height() {
                                panic!("Not enough space in the text atlas!"); // Handle gracefully as needed
                            }


                            let mut new_image = DynamicImage::new( image.placement.width, image.placement.height, image::ColorType::Rgba8);
                            
                            // Place the glyph into the text_atlas
                            for y in 0..glyph_height {
                                for x in 0..glyph_width {
                                    let alpha = image.data[(y as usize * image.placement.width as usize) + x as usize];
                                    self.text_atlas.put_pixel(x + self.x_offset, y + self.y_offset, image::Rgba([alpha, alpha, alpha, alpha]));
                                    new_image.put_pixel(x, y, image::Rgba([alpha, alpha, alpha, alpha]));
                                }
                            }
                            
                            
                            println!("x offset {} y offset {}, glyph id: {}", self.x_offset, self.y_offset, glyph.glyph_id);

                            new_image.save(format!("text_atlas{}-{},{}.png",glyph.glyph_id, image.placement.left, image.placement.top)).unwrap();
                            // Update the x_offset for the next glyph
                            self.x_offset += glyph_width;
                        }
                    }
                }


                self.text_atlas.save("text_atlas.png").unwrap();
                
            } else {
                panic!("Unknown state provided to the renderer!");
            }
        }


    }

    pub(crate) fn draw(&mut self, render_pass: &mut RenderPass) {
    }
}

#[derive(Debug)]
pub struct Glyph {
    x: f32,
    y: f32,
    id: u32,
}

struct BufferGlyphRun {
    font: ID,
    glyphs: Vec<Glyph>,
    glyph_color: Color,
}

pub struct BufferLine {
    glyph_highlights: Vec<Rectangle>,
    cursor: Option<Rectangle>,
    glyph_runs: Vec<BufferGlyphRun>,
}

pub struct BufferGlyphs {
    font_size: f32,
    glyph_highlight_color: Color,
    cursor_color: Color,
    buffer_lines: Vec<BufferLine>,
}

fn create_glyphs(buffer: &Buffer, text_color: Color) -> BufferGlyphs {
    // Get the laid out glyphs and convert them to Glyphs for wgpu.
    let mut last_font: Option<(ID, Color)> = None;

    let mut buffer_glyphs = BufferGlyphs {
        font_size: buffer.metrics().font_size,
        glyph_highlight_color: Color::rgba(255, 0, 0, 50),
        cursor_color: Color::BLACK,
        buffer_lines: vec![],
    };

    for layout_run in buffer.layout_runs() {
        let mut current_glyphs: Vec<Glyph> = vec![];
        let line_i = layout_run.line_i;
        let line_y = layout_run.line_y as f64;
        let line_top = layout_run.line_top as f64;
        let line_height = layout_run.line_height as f64;

        let mut buffer_line = BufferLine {
            glyph_highlights: vec![],
            cursor: None,
            glyph_runs: vec![],
        };

        for glyph in layout_run.glyphs {
            let mut glyph_color = match glyph.color_opt {
                Some(color) => Color::rgba(color.r(), color.g(), color.b(), color.a()),
                None => text_color,
            };

            if let Some((last_font, last_glyph_color)) = last_font {
                if last_font != glyph.font_id || last_glyph_color != glyph_color {
                    buffer_line.glyph_runs.push(BufferGlyphRun {
                        font: last_font,
                        glyphs: current_glyphs,
                        glyph_color: last_glyph_color,
                    });
                    current_glyphs = vec![];
                }
            }

            last_font = Some((glyph.font_id, glyph_color));
            current_glyphs.push(Glyph {
                x: glyph.x,
                y: glyph.y + line_y as f32,
                id: glyph.glyph_id as u32,
            });
        }
        if !current_glyphs.is_empty() {
            let (last_font, last_color) = last_font.unwrap();
            buffer_line.glyph_runs.push(BufferGlyphRun {
                font: last_font,
                glyphs: current_glyphs,
                glyph_color: last_color,
            });
        }

        buffer_glyphs.buffer_lines.push(buffer_line);
    }

    buffer_glyphs
}