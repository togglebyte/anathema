use std::path::Path;
use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferUsages, Device, Features, Instance, InstanceDescriptor, LoadOp, PipelineLayout,
    PipelineLayoutDescriptor, Queue, StoreOp, Surface, Trace,
};
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;

use super::error::{Error, Result};
use super::maths::Size;
use super::model::{INDICES, MODEL};
use crate::wgpu::material::{Material, Materials};
use crate::wgpu::texture::{TextureId, Textures};

pub struct GraphicsCtx {
    pub(crate) window: Arc<Window>,

    pub(crate) device: Device,
    pub(crate) surface: Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pub(crate) queue: Queue,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    pub(crate) textures: Textures,
    pub(crate) materials: Materials,
    pub(crate) pipeline_layout: PipelineLayout,
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
                label: Some("device descriptor"),
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
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&MODEL),
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&INDICES),
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

        let textures = Textures::new(&device);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &textures.bind_group_layout,
                // TODO: add the camera
                // &camera_bindgroupd_layout
            ],
            push_constant_ranges: &[],
        });

        let materials = Materials::new(&device, &pipeline_layout, format);

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
        };

        Ok(inst)
    }

    fn resize(&mut self, size: Size) {
        self.surface_config.width = size.width as u32;
        self.surface_config.height = size.height as u32;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn load_texture(&mut self, path: impl AsRef<Path>) -> TextureId {
        self.textures
            .load_texture(path, &self.device, &self.queue, &mut self.materials)
    }
}
