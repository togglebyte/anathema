use anathema_store::slab::{Slab, SlabIndex};

use crate::wgpu::model::Vertex;
use crate::wgpu::sprite::{SpriteData, SpriteId};
use crate::wgpu::texture::TextureId;
use crate::wgpu::DEFAULT_SHADER;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct MaterialId(u16);

impl From<MaterialId> for u16 {
    fn from(value: MaterialId) -> Self {
        value.0
    }
}

impl SlabIndex for MaterialId {
    const MAX: usize = u16::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u16)
    }
}

pub struct Materials {
    inner: Slab<MaterialId, Material>,
}

impl Materials {
    pub fn new(device: &wgpu::Device, pipeline_layout: &wgpu::PipelineLayout, format: wgpu::TextureFormat) -> Self {
        let mut inst = Self { inner: Slab::empty() };

        let source = wgpu::ShaderSource::Wgsl(DEFAULT_SHADER.into());
        let id = inst.add_material(device, pipeline_layout, source, format, "default".into());
        assert_eq!(id, MaterialId::default());

        inst
    }

    pub(crate) fn get(&self, material: MaterialId) -> &Material {
        &self.inner[material]
    }

    pub(crate) fn get_mut(&mut self, material: MaterialId) -> &mut Material {
        &mut self.inner[material]
    }

    pub(crate) fn add_sprite(&mut self, material: MaterialId, sprite: SpriteId) {
        self.inner
            .get_mut(material)
            .map(|material| material.sprites.push(sprite));
    }

    pub(crate) fn remove_sprite(&mut self, material: MaterialId, sprite: SpriteId) {
        self.inner
            .get_mut(material)
            .map(|material| material.sprites.retain(|id| sprite.ne(id)));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Material> {
        self.inner.iter_values()
    }

    pub(crate) fn add_material(
        &mut self,
        device: &wgpu::Device,
        render_pipeline_layout: &wgpu::PipelineLayout,
        source: wgpu::ShaderSource<'_>,
        format: wgpu::TextureFormat,
        name: String,
    ) -> MaterialId {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout(), SpriteData::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.inner.insert(Material {
            name,
            pipeline,
            sprites: vec![],
        })
    }
}

#[derive(Debug)]
pub struct Material {
    name: String,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) sprites: Vec<SpriteId>,
}
