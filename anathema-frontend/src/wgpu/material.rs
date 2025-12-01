use anathema_store::slab::{Slab, SlabIndex};
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::wgpu::model::Vertex;
use crate::wgpu::sprite::{SpriteData, SpriteId};
use crate::wgpu::texture::TextureId;
use crate::wgpu::DEFAULT_SHADER;

pub type MaterialId = SmallIndex;

pub struct Materials {
    inner: SmallMap<String, Material>,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    texture_format: wgpu::TextureFormat,
}

impl Materials {
    pub fn new(
        device: &wgpu::Device,
        pipeline_layout: wgpu::PipelineLayout,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let mut inst = Self {
            inner: SmallMap::empty(),
            pipeline_layout,
            texture_format,
        };

        let source = wgpu::ShaderSource::Wgsl(DEFAULT_SHADER.into());
        let id = inst.add_material(device, source, "default".into());
        assert_eq!(id, MaterialId::ZERO);

        inst
    }

    pub(crate) fn get(&self, material: MaterialId) -> &Material {
        &self.inner[material]
    }

    pub(crate) fn get_mut(&mut self, material: MaterialId) -> &mut Material {
        &mut self.inner[material]
    }

    pub(crate) fn get_id_by_name(&self, mat: &str) -> Option<MaterialId> {
        self.inner.get_index(mat)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Material> {
        self.inner.iter().map(|(_, material)| material)
    }

    pub(crate) fn add_material(
        &mut self,
        device: &wgpu::Device,
        source: wgpu::ShaderSource<'_>,
        name: String,
    ) -> MaterialId {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&self.pipeline_layout),
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
                    format: self.texture_format,
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

        self.inner.set(name, Material { pipeline })
    }
}

#[derive(Debug)]
pub struct Material {
    pub(crate) pipeline: wgpu::RenderPipeline,
}
