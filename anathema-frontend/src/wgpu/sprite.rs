use std::time::Duration;

use anathema_geometry::{Pos, Size};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, Device, VertexAttribute, VertexBufferLayout, VertexStepMode};

// TODO:
// * Store rotation, scale and translation as separate fields on the Sprite
// * Compute and cache a matrix on the sprite
// * Do NOT send the entire Sprite, but rather the computed matrix for the sprite

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SpriteData {
    uv_size: Size,
    offset: Pos,
    mat: Mat4,
}

impl SpriteData {
    pub fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 6] = wgpu::vertex_attr_array![
            5 => Float32x2,
            6 => Float32x2,
            7 => Float32x4,
            8 => Float32x4,
            9 => Float32x4,
            10 => Float32x4,
        ];

        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }

    fn new(uv_size: Size, offset: Pos) -> Self {
        Self {
            mat: Mat4::IDENTITY,
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
        let offset = Pos::from(*offset.into() * pixel_size.to_vec());

        Self {
            pos: Pos::ZERO,
            rot: 0.0,
            scale: 1.0,
            offset,
            size_in_pixels,

            dirty: true,
            cache: SpriteData::new(uv_size, offset),
            pixel_size,
        }
    }

    pub fn data(&mut self) -> SpriteData {
        if self.dirty {
            let translation = Mat4::from_translation(Vec3::from((*self.pos, 0.0)));
            let rotation = Mat4::from_rotation_z(self.rot);

            // here we scale pixels
            let pixel_size_transform = Mat4::from_scale(Vec3::from((self.size_in_pixels.to_vec(), 1.0)));

            let scale = Size::new(self.scale, self.scale);
            let uniform_scale = Mat4::from_scale(Vec3::from((scale.to_vec(), 1.0)));

            let mat_row_major = translation * rotation * uniform_scale * pixel_size_transform;

            // self.cache.mat = mat_row_major;//.transpose();
            self.cache.mat = Mat4::from_cols(Vec4::X, Vec4::Y, Vec4::Z, Vec4::W);
            self.cache.offset = (*self.offset * self.pixel_size.to_vec()).into();

            // Brutalise the data because f this
            self.cache.offset = Pos::ZERO;
            self.cache.uv_size = Size::ONE;
            self.cache.mat = Mat4::IDENTITY;
        }

        self.cache
    }

    pub fn pos(&self) -> Pos {
        self.pos
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
    pub(crate) sprites: Vec<Sprite>,
    pub(crate) sprite_cache: Vec<SpriteData>,
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

    pub(crate) fn rebuild_buffer(&mut self, device: &Device) {
        self.sprite_cache.clear();
        self.sprites
            .iter_mut()
            .map(|sprite| sprite.data())
            .for_each(|data| self.sprite_cache.push(data));
        self.len = self.sprites.len();

        let debug_instance_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Debug instance buffer"),
            contents: bytemuck::cast_slice(&[self.sprite_cache[0]]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        self.instance_buffer = debug_instance_buf;

        // self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("Instance buffer"),
        //     contents: bytemuck::cast_slice(&self.sprite_cache),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        // });
    }

    pub fn add(&mut self, mut sprite: Sprite, device: &Device) {
        self.sprite_cache.push(sprite.data());
        self.sprites.push(sprite);
        self.rebuild_buffer(device);

        if self.sprites.len() >= self.len {
            self.rebuild_buffer(device);
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.len as u32
    }
}
