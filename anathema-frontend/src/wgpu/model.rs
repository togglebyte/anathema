use anathema_geometry::Pos;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct Vertex {
    pos: Pos,
    texture_coords: Pos,
}

impl Vertex {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub(crate) static MODEL: &[Vertex] = &[
    // top left
    Vertex {
        pos: Pos::new(-0.5, 0.5),
        texture_coords: Pos::ZERO,
    },
    // // bottom left
    Vertex {
        pos: Pos::new(-0.5, -0.5),
        texture_coords: Pos::new(0.0, 1.0),
    },
    // // bottom right
    Vertex {
        pos: Pos::new(0.5, -0.5),
        texture_coords: Pos::new(1.0, 1.0),
    },
    // // top right
    Vertex {
        pos: Pos::new(0.5, 0.5),
        texture_coords: Pos::new(1.0, 0.0),
    },
];

pub(crate) static INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
pub(crate) static INDEX_COUNT: u32 = INDICES.len() as u32;
