use std::time::Duration;

use anathema_geometry::{Mat4x4, Pos, Size};
use bytemuck::{Pod, Zeroable};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, Device, VertexAttribute, VertexBufferLayout, VertexStepMode};

// TODO:
// * Store rotation, scale and translation as separate fields on the Sprite
// * Compute and cache a matrix on the sprite
// * Do NOT send the entire Sprite, but rather the computed matrix for the sprite

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SpriteData {
    // Sprite sheet index
    index: i32,
    mat: Mat4x4,
    uv_size: [f32; 2],
    offset: [f32; 2],
}

impl SpriteData {
    pub fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 7] = wgpu::vertex_attr_array![4 => Sint32, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Float32x2, 10 => Float32x2];

        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }

    fn new(index: i32, uv_size: [f32; 2], offset: [f32; 2]) -> Self {
        Self {
            index: 2,
            mat: Mat4x4::identity(),
            uv_size,
            offset,
        }
    }
}

pub struct Sprite {
    pos: Pos,
    pub scale: f32,
    rot: f32,

    // Size, in pixels
    pub size_in_pixels: Size,
    // Offset, in pixels
    pub offset: Pos,
    // // Z index
    // pub z_index: f32,
    pub index: i32,
    dirty: bool,
    pixel_size: Size,

    cache: SpriteData,
}

impl Sprite {
    pub fn new(sheet_size: impl Into<Size>, offset: impl Into<Pos>, size_in_pixels: impl Into<Size>) -> Self {
        let sheet_size = sheet_size.into();
        let pixel_size = Size::ONE / sheet_size;
        let size_in_pixels = size_in_pixels.into();
        let uv_size = size_in_pixels * pixel_size;
        let offset = offset.into() * pixel_size.to_vec();

        Self {
            pos: Pos::ZERO,
            rot: 0.0,
            scale: 1.0,
            offset,
            size_in_pixels,

            dirty: true,
            cache: SpriteData::new(0, uv_size.into(), offset.into()),
            index: 0,
            pixel_size,
        }
    }

    pub fn data(&mut self) -> SpriteData {
        if self.dirty {
            let translation = Mat4x4::from_translation(self.pos.to_vec());
            let rotation = Mat4x4::from_rotation(self.rot);

            // here we scale pixels
            let pixel_size_tranform = Mat4x4::from_scale(self.size_in_pixels);

            let uniform_scale = Mat4x4::from_scale(Size::new(self.scale, self.scale));

            self.cache.mat = translation * rotation * uniform_scale * pixel_size_tranform;
            self.cache.index = self.index;
            self.cache.offset = (self.offset * self.pixel_size.to_vec()).into();
        }

        self.cache
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn index(&mut self, index: i32) {
        self.index = index;
        self.dirty = true;
    }

    pub fn translate(&mut self, pos: Pos) {
        self.pos = pos;
        self.dirty = true;
    }

    pub fn rotate(&mut self, rad: f32) {
        self.rot = rad;
        self.dirty = true;
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale = scale;
        self.dirty = true;
    }
}

pub(crate) struct Sprites {
    sprites: Vec<Sprite>,
    sprite_cache: Vec<SpriteData>,
    pub(crate) instance_buffer: wgpu::Buffer,
    len: usize,
}

impl Sprites {
    pub fn new(device: &Device) -> Self {
        let sprite_cache = vec![];
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&sprite_cache),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            sprites: vec![],
            sprite_cache,
            instance_buffer,
            len: 0,
        }
    }

    fn rebuild_buffer(&mut self, device: &Device) {
        self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&self.sprite_cache),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
    }

    pub fn add(&mut self, mut sprite: Sprite, device: &Device) {
        self.sprite_cache.push(sprite.data());
        self.sprites.push(sprite);

        if self.sprites.len() > self.len {
            self.rebuild_buffer(device);
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.len as u32
    }
}
