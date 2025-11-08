use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

use anathema_geometry::{Pos, Size};
use crate::wgpu::maths::Mat;

// TODO:
// * Store rotatio, scale and translation as separate fields on the Sprite
// * Compute and cache a matrix on the sprite
// * Do NOT send the entire Sprite, but rather the computed matrix for the sprite

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SpriteData {
    // Sprite sheet index
    index: i32,
    mat: Mat,
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
            mat: Mat::identity(),
            uv_size,
            offset,
        }
    }
}

pub struct Sprite {
    pos: Pos,
    // Scale (whatever nerd)
    pub scale: f32,
    // Rotation (like in toast?)
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
    pub fn new(
        sheet_size: impl Into<Size>,
        offset: impl Into<Pos>,
        size_in_pixels: impl Into<Size>,
    ) -> Self {
        let sheet_size = sheet_size.into();
        let pixel_size = Size::ONE / sheet_size;
        let size_in_pixels = size_in_pixels.into();
        let uv_size = size_in_pixels * pixel_size;
        let offset = offset.into() * pixel_size;

        Self {
            pos: Pos::ZERO,
            rot: 0.0,
            scale: 1.0,
            offset,
            size_in_pixels,

            dirty: true,
            cache: SpriteData::new(0, uv_size.into(), offset.into()),
            index: 0,
            animation: None,
            pixel_size,
        }
    }

    pub fn data(&mut self) -> SpriteData {
        if self.dirty {
            let translation = Mat4x4::from_translation(self.pos);
            let rotation = Mat4x4::from_rotation(self.rot);

            // here we scale pixels
            let pixel_size_tranform = Mat4x4::from_scale(self.size_in_pixels);

            let uniform_scale = Mat4x4::from_scale(Size::new(self.scale, self.scale));

            self.cache.mat = translation * rotation * uniform_scale * pixel_size_tranform;
            self.cache.index = self.index;
            self.cache.offset = (self.offset * self.pixel_size).into();
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

    pub(crate) fn update(&mut self, dt: Duration) {
        let Some(anim) = self.animation.as_mut() else { return };
        self.offset =  anim.update(dt);
        self.dirty = true;
    }
}
