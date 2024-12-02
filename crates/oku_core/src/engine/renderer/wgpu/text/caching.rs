use std::collections::HashMap;
use std::path::Path;
use cosmic_text::{CacheKey, Placement, SwashContent, SwashImage};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat};

#[derive(Clone)]
pub struct GlyphInfo {
    pub(crate) texture_coordinate_x: u32,
    pub(crate) texture_coordinate_y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub swash_image_placement: Placement,
    pub(crate) content_type: u32
}

pub struct TextAtlas {
    texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) texture_sampler: wgpu::Sampler,
    texture_width: u32,
    texture_height: u32,
    glyph_cache: HashMap<CacheKey, GlyphInfo>,
    x_offset: u32,
    y_offset: u32,
    tallest_glyph_on_current_row: u32,
}

impl TextAtlas {

    pub(crate) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        
        // FIXME: Do not hardcode this.
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Atlas Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        TextAtlas {
            texture,
            texture_view: view,
            texture_sampler: sampler,
            texture_width: width,
            texture_height: height,
            glyph_cache: Default::default(),
            x_offset: 0,
            y_offset: 0,
            tallest_glyph_on_current_row: 0,
        }
    }

    pub(crate) fn get_cached_glyph_info(&self, cache_key: CacheKey) -> Option<GlyphInfo> {
        self.glyph_cache.get(&cache_key).cloned()
    }
    
    fn set_cached_glyph_info(&mut self, cache_key: CacheKey, glyph_info: GlyphInfo) {
        self.glyph_cache.insert(cache_key, glyph_info);
    }


    pub(crate) fn add_glyph(&mut self, swash_image: &SwashImage, cache_key: CacheKey, queue: &wgpu::Queue) {
        if swash_image.placement.height == 0 {
            return;
        }

        let glyph_width = swash_image.placement.width;
        let glyph_height = swash_image.placement.height;

        self.tallest_glyph_on_current_row = self.tallest_glyph_on_current_row.max(glyph_height);
        
        // Check if the glyph fits in the current row.
        if self.x_offset + glyph_width > self.texture_width {
            // Move to the next row.
            self.x_offset = 0;
            self.y_offset += self.tallest_glyph_on_current_row; // Adjust as necessary based on your glyph heights
            self.tallest_glyph_on_current_row = glyph_height;
        }

        // Ensure we don't exceed the atlas height.
        if self.y_offset + glyph_height > self.texture_height {
            panic!("Not enough space in the text atlas!"); // Handle gracefully as needed
        }

        // Place the glyph into the text_atlas.

        let mut data: Vec<u8> = vec![0; (glyph_width * glyph_height * 4) as usize];
        let mut content_type;
        
        let data = match swash_image.content {
            SwashContent::Mask => {
                content_type = 0;
                
                let mut data_i = 0;
                for y in 0..glyph_height {
                    for x in 0..glyph_width {
                        let alpha = swash_image.data[(y as usize * swash_image.placement.width as usize) + x as usize];
                        // self.text_atlas.put_pixel(x + self.x_offset, y + self.y_offset, image::Rgba([alpha, alpha, alpha, alpha]));
                        data[data_i] = 0xFF;
                        data[data_i + 1] = 0xFF;
                        data[data_i + 2] = 0xFF;
                        data[data_i + 3] = alpha;
                        data_i += 4;
                    }
                }

                data.as_slice()
            }
            SwashContent::Color => {
                content_type = 1;
                &swash_image.data
            }
            SwashContent::SubpixelMask => {
                panic!("Subpixel mask not yet implemented!");
            }
        };

        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: self.x_offset,
                    y: self.y_offset,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(glyph_width * 4),
                rows_per_image: None,
            },
            Extent3d {
                width: glyph_width,
                height: glyph_height,
                depth_or_array_layers: 1,
            },
        );

        self.set_cached_glyph_info(cache_key, GlyphInfo {
            texture_coordinate_x: self.x_offset,
            texture_coordinate_y: self.y_offset,
            width: glyph_width,
            height: glyph_height,
            swash_image_placement: swash_image.placement,
            content_type,
        });
        
        // Update the x_offset for the next glyph.
        self.x_offset += glyph_width;
    }

    fn save_atlas_to_file(&self, file_path: &Path) {
        /*
        
        
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
        */
    }
}