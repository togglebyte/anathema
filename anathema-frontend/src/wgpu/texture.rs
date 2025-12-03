use std::num::NonZeroU32;
use std::path::Path;

use anathema_geometry::Size;
use anathema_store::gen_key;
use anathema_store::slab::{Basic, SlabIndex};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, FilterMode, Origin3d, Queue, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TexelCopyTextureInfoBase, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::wgpu::material::{MaterialId, Materials};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureId(u32);

impl From<TextureId> for u32 {
    fn from(value: TextureId) -> Self {
        value.0
    }
}

impl SlabIndex for TextureId {
    const MAX: usize = u32::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
    }
}

#[derive(Debug)]
pub struct Texture {
    inner: wgpu::Texture,
    pub(crate) bind_group: BindGroup,
}

pub(super) struct Textures {
    inner: Basic<TextureId, Texture>,
    pub(crate) bind_group_layout: BindGroupLayout,
    diffuse_sampler: Sampler,
}

impl Textures {
    pub(crate) fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            inner: Basic::empty(),
            bind_group_layout,
            diffuse_sampler,
        }
    }

    pub(crate) fn remove_texture(&mut self, id: TextureId) {
        let _texture = self.inner.remove(id);
    }

    pub(crate) fn load_texture(&mut self, path: impl AsRef<Path>, device: &Device, queue: &Queue) -> (TextureId, Size) {
        let path = path.as_ref().to_str().unwrap_or("<path>");
        let diffuse_bytes = std::fs::read(path).unwrap();
        self.load_texture_bytes(&diffuse_bytes, device, queue, path)
    }

    pub(crate) fn load_texture_bytes(&mut self, bytes: &[u8], device: &Device, queue: &Queue, name: &str) -> (TextureId, Size) {
        let (texture, size) = self.load_single_texture(bytes, device, queue);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let bind_group = self.single_texture_bind_group(device, &view, name);
        let texture = self.inner.insert(Texture {
            inner: texture,
            bind_group,
        });

        (texture, size)
    }

    fn load_single_texture(&mut self, diffuse_bytes: &[u8], device: &Device, queue: &Queue) -> (wgpu::Texture, Size) {
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let (width, height) = diffuse_image.dimensions();

        let texture_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &diffuse_rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        (diffuse_texture, Size::new(width as f32, height as f32))
    }

    fn single_texture_bind_group(&mut self, device: &Device, texture: &TextureView, path: &str) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Texture bind group: {path}")),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.diffuse_sampler),
                },
            ],
        })
    }

    pub(crate) fn get(&self, id: TextureId) -> &Texture {
        &self.inner[id]
    }
}
