use std::ops::Index;

use anathema_geometry::{CharacterPos, Pos, ScreenPos, Size};
use anathema_hashmap::HashMap;
use anathema_store::slab::{SecondaryMap, SparseSlab};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

use crate::wgpu::material::{MaterialGroups, Materials};
use crate::wgpu::texture::TextureId;
use crate::wgpu::{MaterialId, Sprite};

pub(crate) static DEFAULT_FONT: &'static [u8] = include_bytes!("font.png");
pub(crate) static DEFAULT_FONT_SHADER: &'static str = include_str!("fontshader.wgsl");

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Char {
    pixel_offset: Pos,
    pos: CharacterPos,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct FontData {
    translation: Mat4,
    size: Vec2,
    _padding: i64,
}

impl FontData {
    fn new(translation: Mat4, size: Vec2) -> Self {
        Self {
            translation,
            size,
            _padding: 0,
        }
    }
}

pub struct Font {
    pub(crate) texture: TextureId,
    // Size of the texture in pixels
    texture_size: Size,
    // Size of a character in pixels
    char_size: Size,
    pub(crate) buffer: Buffer,
    pub(crate) bind_group: BindGroup,
    chars: MaterialGroups<Char>,
}

impl Font {
    fn new(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        texture: TextureId,
        texture_size: Size,
        char_size: Size,
    ) -> Self {
        let font_data = FontData::new(Mat4::IDENTITY, char_size.to_vec());
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Font buffer"),
            contents: bytemuck::cast_slice(&[font_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font bind group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Font {
            texture,
            texture_size,
            char_size,
            buffer,
            bind_group,
            chars: panic!(),
        }
    }

    fn clear_material_groups(&mut self) {
        self.chars.for_each(|_material, chars| chars.clear());
    }

    // This is the uniform value that is uploaded to the shader
    pub(crate) fn data(&self) -> FontData {
        let translation = Mat4::IDENTITY;

        // // here we scale pixels
        // let pixel_size_transform = Mat4::from_scale(Vec3::from((self.size_in_pixels.to_vec(), 1.0)));

        // let uniform_scale = Mat4::from_scale(Vec3::from((self.scale.to_vec(), 1.0)));

        // let mat = translation * rotation * uniform_scale * pixel_size_transform;
        // let offset = (*self.offset * self.pixel_size.to_vec()).into();

        // SpriteData {
        //     uv_size: self.uv_size,
        //     mat,
        //     offset,
        // }

        FontData::new(translation, self.char_size.to_vec())
    }

    pub(crate) fn get_offset(&self, c: char) -> Pos {
        let pos = match c {
            ' ' => Pos::ZERO,
            'a' => Pos::new(1.0, 0.0),
            'b' => Pos::new(2.0, 0.0),
            'c' => Pos::new(3.0, 0.0),
            'd' => Pos::new(4.0, 0.0),
            'e' => Pos::new(5.0, 0.0),
            'f' => Pos::new(6.0, 0.0),
            'g' => Pos::new(7.0, 0.0),
            'h' => Pos::new(8.0, 0.0),
            'i' => Pos::new(9.0, 0.0),
            'j' => Pos::new(10.0, 0.0),
            'k' => Pos::new(11.0, 0.0),
            'l' => Pos::new(12.0, 0.0),
            'm' => Pos::new(13.0, 0.0),
            'n' => Pos::new(14.0, 0.0),
            'o' => Pos::new(15.0, 0.0),
            _missing => Pos::new(0.0, 1.0),
        };

        pos * self.char_size
    }
}

pub(crate) struct Fonts {
    inner: Vec<Font>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Fonts {
    pub fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Font bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        Self {
            inner: vec![],
            bind_group_layout,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Font> {
        self.inner.iter_mut()
    }

    pub fn new_font(&mut self, device: &Device, texture: TextureId, size: Size, char_size: Size, material: MaterialId) {
        let font = Font::new(device, &self.bind_group_layout, texture, size, char_size);

        self.inner.push(font);
    }

    pub(crate) fn character(&self, c: char, pos: CharacterPos) -> Char {
        let pixel_offset = self.inner[0].get_offset(c);
        Char { pos, pixel_offset }
    }

    pub(crate) fn clear_material_groups(&mut self) {
        self.inner.iter_mut().for_each(|font| font.clear_material_groups());
    }
}
