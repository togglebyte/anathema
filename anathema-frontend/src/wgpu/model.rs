use bytemuck::{Pod, Zeroable};

use super::maths::Pos2d;

#[derive(Debug, Copy, Clone, Zeroable)]
pub(crate) struct Vertex {
    pos: Pos2d,
    texture_coords: Pos2d,
}

unsafe impl Pod for Vertex {
}

impl Vertex {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub(crate) static MODEL: &[Vertex] = &[
    // Top left
    Vertex {
        pos: Pos2d::from_x_y(-0.5, 0.5),
        texture_coords: Pos2d::ZERO,
    },
    // Bottom left
    Vertex {
        pos: Pos2d::from_x_y(-0.5, -0.5),
        texture_coords: Pos2d::from_x_y(0.0, 1.0),
    },
    // Top right
    Vertex {
        pos: Pos2d::from_x_y(0.5, 0.5),
        texture_coords: Pos2d::from_x_y(1.0, 1.0),
    },
    // Bottom right
    Vertex {
        pos: Pos2d::from_x_y(0.5, -0.5),
        texture_coords: Pos2d::from_x_y(1.0, 0.0),
    },
];

pub(crate) static INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
