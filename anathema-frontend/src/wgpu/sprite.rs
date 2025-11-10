use std::hash::Hash;
use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_hashmap::HashMap;
use anathema_store::slab::{Slab, SlabIndex};
use anathema_store::stack::Stack;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, Device, VertexAttribute, VertexBufferLayout, VertexStepMode};

use crate::wgpu::texture::TextureId;
use crate::wgpu::MaterialId;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpriteId(u32);

impl SlabIndex for SpriteId {
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
    pub(crate) texture: TextureId,
    pub(crate) material: MaterialId,
    cache_index: usize,

    pos: Pos,
    pub scale: f32,
    rot: f32,
    uv_size: Size,

    // Size, in pixels
    pub size_in_pixels: Size,
    // Offset, in pixels
    pub offset: Pos,
    // // Z index
    // pub z_index: f32,
    dirty: bool,
    pixel_size: Size,
    // cache: SpriteData,
}

impl Sprite {
    pub fn new(
        texture: TextureId,
        material: MaterialId,
        sheet_size: impl Into<Size>,
        offset: impl Into<Pos>,
        size_in_pixels: impl Into<Size>,
    ) -> Self {
        let sheet_size = sheet_size.into();
        let pixel_size = Size::ONE / sheet_size;
        let size_in_pixels = size_in_pixels.into();
        let uv_size = size_in_pixels * pixel_size;
        let offset = Pos::from(*offset.into() * pixel_size.to_vec());

        Self {
            texture,
            material,
            cache_index: usize::MAX,

            pos: Pos::ZERO,
            rot: 0.0,
            scale: 1.0,
            offset,
            size_in_pixels,
            uv_size,

            dirty: true,
            // cache: SpriteData::new(uv_size, offset),
            pixel_size,
        }
    }

    pub fn data(&mut self) -> SpriteData {
        let translation = Mat4::from_translation(Vec3::from((*self.pos, 0.0)));
        let rotation = Mat4::from_rotation_z(self.rot);

        // here we scale pixels
        let pixel_size_transform = Mat4::from_scale(Vec3::from((self.size_in_pixels.to_vec(), 1.0)));

        let scale = Size::new(self.scale, self.scale);
        let uniform_scale = Mat4::from_scale(Vec3::from((scale.to_vec(), 1.0)));

        let mat = translation * rotation * uniform_scale * pixel_size_transform;
        let offset = (*self.offset * self.pixel_size.to_vec()).into();

        SpriteData {
            uv_size: self.uv_size,
            mat,
            offset,
        }
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
    pub(crate) sprites: Slab<SpriteId, Sprite>,
    pub(crate) sprite_cache: SpriteBuffers,
}

impl Sprites {
    pub fn new(device: &Device) -> Self {
        let sprite_cache = SpriteBuffers::empty();

        Self {
            sprites: Slab::empty(),
            sprite_cache,
        }
    }

    pub(crate) fn get(&self, id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(id)
    }

    pub fn add(&mut self, mut sprite: Sprite, device: &Device) -> SpriteId {
        let key = Key(sprite.texture, sprite.material);
        sprite.cache_index = self.sprite_cache.insert(key, sprite.data(), device);
        let sprite_id = self.sprites.insert(sprite);
        sprite_id
    }

    pub(crate) fn iter_cache(&self) -> impl Iterator<Item = (TextureId, MaterialId, &wgpu::Buffer, u32)> {
        self.sprite_cache.iter()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Key(TextureId, MaterialId);

impl Key {
    pub(crate) fn consume(self) -> (TextureId, MaterialId) {
        (self.0, self.1)
    }
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let texture = u32::from(self.0) as u64;
        let material = u16::from(self.1) as u64;
        let key = texture | (material << 32);
        state.write_u64(key);
    }
}

#[derive(Debug)]
struct BufferEntry {
    data: Vec<SpriteData>,
    buffer: wgpu::Buffer,
}

pub struct SpriteBuffers {
    inner: HashMap<Key, BufferEntry>,
}

impl SpriteBuffers {
    fn insert(&mut self, key: Key, data: SpriteData, device: &wgpu::Device) -> usize {
        match self.inner.entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let buffer_entry = entry.into_mut();
                let index = buffer_entry.data.len();
                buffer_entry.data.push(data);

                // write to the buffer (we aren't writing, we are replacing, change this to write)
                let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Instance buffer"),
                    contents: bytemuck::cast_slice(buffer_entry.data.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

                index
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let sprite_data = vec![data];

                let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Instance buffer"),
                    contents: bytemuck::cast_slice(sprite_data.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

                let buffer_entry = BufferEntry {
                    data: sprite_data,
                    buffer: instance_buffer,
                };

                entry.insert(buffer_entry);
                0
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = (TextureId, MaterialId, &wgpu::Buffer, u32)> {
        self.inner.iter().map(|(key, entry)| {
            let (tex, mat) = key.consume();
            (tex, mat, &entry.buffer, entry.data.len() as u32)
        })
    }

    pub(crate) fn rebuild_buffer(&mut self, device: &Device) {
        panic!()
        // self.sprite_cache.clear();
        // self.sprites
        //     .iter_mut()
        //     .map(|(_, sprite)| sprite.data())
        //     .for_each(|data| self.sprite_cache.push(data));

        // self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("Instance buffer"),
        //     contents: bytemuck::cast_slice(&self.sprite_cache),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        // });
    }

    fn empty() -> Self {
        Self {
            inner: HashMap::default(),
        }
    }
}
