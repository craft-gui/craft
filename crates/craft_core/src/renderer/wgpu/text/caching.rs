use crate::renderer::wgpu::texture::Texture;
use cosmic_text::{CacheKey, Placement, SwashContent, SwashImage};
use std::collections::HashMap;
use wgpu::{BindGroup, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect};

#[repr(u8)]
#[derive(Clone)]
pub enum ContentType {
    Mask = 0,
    // This is for emojis.
    Color = 1, 
    // For the cursor and highlights.
    Rectangle = 2,
}

#[derive(Clone)]
pub struct GlyphInfo {
    pub(crate) texture_coordinate_x: u32,
    pub(crate) texture_coordinate_y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub swash_image_placement: Placement,
    pub(crate) content_type: ContentType
}

pub struct TextAtlas {
    texture: wgpu::Texture,
    pub(crate) _texture_view: wgpu::TextureView,
    pub(crate) _texture_sampler: wgpu::Sampler,
    pub(crate) texture_bind_group: BindGroup,
    pub(crate) texture_width: u32,
    pub(crate) texture_height: u32,
    glyph_cache: HashMap<CacheKey, GlyphInfo>,
    x_offset: u32,
    y_offset: u32,
    tallest_glyph_on_current_row: u32,
}

impl TextAtlas {

    pub(crate) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let max_texture_size = device.limits().max_texture_dimension_2d;
        let texture_width = u32::clamp(width, 1, max_texture_size);
        let texture_height = u32::clamp(height, 1, max_texture_size);
        
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Atlas Texture"),
            size: Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::DEFAULT_FORMAT,
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

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("craft_bind_group"),
        });
        
        TextAtlas {
            texture,
            _texture_view: view,
            _texture_sampler: sampler,
            texture_bind_group,
            texture_width,
            texture_height,
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
        let content_type;
        
        let data = match swash_image.content {
            SwashContent::Mask => {
                content_type = ContentType::Mask;
                
                let mut data_i = 0;
                for y in 0..glyph_height {
                    for x in 0..glyph_width {
                        let alpha = swash_image.data[(y as usize * swash_image.placement.width as usize) + x as usize];
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
                content_type = ContentType::Color;
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
            data,
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
}