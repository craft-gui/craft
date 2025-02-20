use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use std::io::Cursor;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub(crate) texture_bind_group: wgpu::BindGroup,
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
 
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: Option<&str>) -> Option<Self> {
        let image = image::load_from_memory(bytes).unwrap().to_rgba8();
        Self::from_image(device, queue, &image, label)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::RgbaImage,
        label: Option<&str>,
    ) -> Option<Self> {
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        
        // Wgpu will panic when creating the texture if the width or height is 0.
        if size.width == 0 || size.height == 0 {
            return None;
        }
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEFAULT_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

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
            label: Some("oku_bind_group"),
        });

        Some(Self {
            texture,
            view,
            sampler,
            texture_bind_group
        })
    }

    pub fn generate_default_white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let white_pixel = image::Rgba([255, 255, 255, 255]);
        let image_buffer = image::ImageBuffer::from_pixel(1, 1, white_pixel);

        let mut cursor = Cursor::new(Vec::new());
        let encoder = PngEncoder::new(&mut cursor);

        // Encode the image as a PNG and write the image bytes into our cursor.
        encoder
            .write_image(image_buffer.as_raw(), 1, 1, ExtendedColorType::Rgba8)
            .expect("Failed to create a default texture");

        Texture::from_bytes(device, queue, cursor.get_mut().as_slice(), Some("1x1_white_texture"))
            .expect("Failed to create the default texture")
    }
}
