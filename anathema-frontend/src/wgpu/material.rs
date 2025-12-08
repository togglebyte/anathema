use anathema_store::secondary_map::{BasicStorage, SecondaryMap};
use anathema_store::slab::{Basic, Sparse};
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::wgpu::model::Vertex;
use crate::wgpu::sprite::{SpriteData, SpriteId};
use crate::wgpu::texture::TextureId;
use crate::wgpu::DEFAULT_SHADER;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaterialId(SmallIndex);

impl MaterialId {
    pub const ZERO: Self = Self(SmallIndex::ZERO);
}

impl Default for MaterialId {
    fn default() -> Self {
        Self(SmallIndex::ZERO)
    }
}

impl From<SmallIndex> for MaterialId {
    fn from(value: SmallIndex) -> Self {
        Self(value)
    }
}

impl From<MaterialId> for SmallIndex {
    fn from(value: MaterialId) -> Self {
        value.0
    }
}

impl From<usize> for MaterialId {
    fn from(value: usize) -> Self {
        Self(value.into())
    }
}

impl From<MaterialId> for usize {
    fn from(value: MaterialId) -> Self {
        value.0.into()
    }
}

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
        &self.inner[material.0]
    }

    pub(crate) fn get_mut(&mut self, material: MaterialId) -> &mut Material {
        &mut self.inner[material.0]
    }

    pub(crate) fn get_id_by_name(&self, mat: &str) -> Option<MaterialId> {
        self.inner.get_index(mat).map(|i| i.into())
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

        self.inner.set(name, Material { pipeline }).into()
    }
}

#[derive(Debug)]
pub struct Material {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

pub(crate) struct MaterialGroups<K, V>(SecondaryMap<BasicStorage<MaterialId, Sparse<K, V>>>);

impl<K, V> MaterialGroups<K, V>
where
    K: Copy,
    K: From<usize>,
    K: PartialEq,
    usize: From<K>,
{
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn get_or_create(&mut self, key: MaterialId) -> &mut Sparse<K, V> {
        panic!()
    }

    pub fn for_each(&mut self, f: impl Fn(MaterialId, &mut Sparse<K, V>)) {
        self.0.for_each(f)
    }
}
