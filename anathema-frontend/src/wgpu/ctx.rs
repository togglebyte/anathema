use std::path::Path;
use std::sync::Arc;

use anathema_geometry::Size;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, Buffer, BufferUsages, Device, Features, Instance,
    InstanceDescriptor, LoadOp, PipelineLayout, PipelineLayoutDescriptor, Queue, StoreOp, Surface, Trace,
};
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;

use super::error::{Error, Result};
use super::model::{INDICES, MODEL};
use crate::wgpu::camera::Camera;
use crate::wgpu::material::{Material, MaterialId, Materials};
use crate::wgpu::sprite::{Sprite, SpriteId, Sprites};
use crate::wgpu::texture::{Texture, TextureId, Textures};

pub const NEAR: f32 = 10.0;
pub const FAR: f32 = -10.0;

pub struct GraphicsCtx {
    pub(crate) window: Arc<Window>,

    pub(crate) device: Device,
    pub(crate) surface: Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pub(crate) queue: Queue,

    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,

    pub(crate) sprites: Sprites,

    pub(crate) textures: Textures,
    pub(crate) materials: Materials,
    pub(crate) pipeline_layout: PipelineLayout,
    pub(crate) camera: Camera,
    pub(crate) projection_buffer: Buffer,
    pub(crate) camera_bind_group: BindGroup,
}

impl GraphicsCtx {
    pub async fn new(window: Window) -> Result<Self> {
        let window = Arc::new(window);

        let instance = Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // -----------------------------------------------------------------------------
        //   - Surface and Adapter -
        // -----------------------------------------------------------------------------
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|_| Error::AdapterUnavailable)?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // -----------------------------------------------------------------------------
        //   - Device and queue -
        // -----------------------------------------------------------------------------
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device descriptor"),
                required_features: Features::TEXTURE_BINDING_ARRAY
                    | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

        // -----------------------------------------------------------------------------
        //   - Buffers -
        // -----------------------------------------------------------------------------
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(MODEL),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: BufferUsages::INDEX,
        });

        // -----------------------------------------------------------------------------
        //   - Surface config -
        // -----------------------------------------------------------------------------
        let size = window.inner_size();
        let (width, height) = (size.width, size.height);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            alpha_mode: surface_caps.alpha_modes[0],
            present_mode: wgpu::PresentMode::Fifo,
            // present_mode: wgpu::PresentMode::Immediate,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        // -----------------------------------------------------------------------------
        //   - Camera -
        // -----------------------------------------------------------------------------
        let camera = Camera::new(Size::new(width as f32, height as f32), NEAR, FAR);
        let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Projection"),
            contents: bytemuck::cast_slice(&[camera.to_matrix()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bindgroupd_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
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

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Projection & Camera bind group"),
            layout: &camera_bindgroupd_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        // -----------------------------------------------------------------------------
        //   - Textures, materials and sprites -
        // -----------------------------------------------------------------------------
        let textures = Textures::new(&device);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&textures.bind_group_layout, &camera_bindgroupd_layout],
            push_constant_ranges: &[],
        });

        let materials = Materials::new(&device, &pipeline_layout, format);
        let sprites = Sprites::new(&device);

        // -----------------------------------------------------------------------------
        //   - Done... -
        // -----------------------------------------------------------------------------
        let inst = Self {
            window,

            surface,
            surface_config,
            device,
            queue,

            vertex_buffer,
            index_buffer,

            textures,
            materials,
            pipeline_layout,
            sprites,

            camera,
            camera_bind_group,
            projection_buffer,
        };

        Ok(inst)
    }

    pub(crate) fn resize(&mut self, size: Size) {
        self.camera.resize(size, NEAR, FAR);
        self.surface_config.width = size.width as u32;
        self.surface_config.height = size.height as u32;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn load_texture(&mut self, path: impl AsRef<Path>) -> TextureId {
        self.textures.load_texture(path, &self.device, &self.queue)
    }

    pub fn load_texture_from_bytes(&mut self, bytes: &[u8], name: &str) -> TextureId {
        self.textures.load_texture_bytes(bytes, &self.device, &self.queue, name)
    }

    pub fn add_sprite(&mut self, sprite: Sprite) -> SpriteId {
        let material = sprite.material;
        let sprite = self.sprites.add(sprite, &self.device);
        self.materials.add_sprite(material, sprite);
        sprite
    }

    pub(crate) fn sprites(&self, sprites: &[SpriteId]) -> impl Iterator<Item = (&Sprite, &Texture)> {
        sprites.iter().filter_map(|id| self.sprites.get(*id)).map(|sprite| {
            let texture = self.textures.get(sprite.texture);
            (sprite, texture)
        })
    }

    // TODO: remove this, it's a test function,
    // but remember to rebuild the buffer!
    pub fn each_sprite<F>(&mut self, f: F)
    where
        F: Fn(&mut Sprite),
    {
        for (_, sprite) in self.sprites.sprites.iter_mut() {
            f(sprite);
            panic!("make a key, update the sprite in the cache");
        }
    }

    pub(crate) fn something(&self) -> impl Iterator<Item = (&Material, &Texture, &Buffer, u32)> {
        self.sprites.iter_cache().map(|(texture, material, buffer, len)| {
            let texture = self.textures.get(texture);
            let material = self.materials.get(material);

            (material, texture, buffer, len)
        })
    }
}
