use memoffset::offset_of;
use once_cell::sync::Lazy;
use wgpu::InputStepMode;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 2]> = Lazy::new(|| {
            [
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                    offset: offset_of!(Vertex, position) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    shader_location: 1,
                    offset: offset_of!(Vertex, tex_coords) as _,
                },
            ]
        });

        wgpu::VertexBufferLayout {
            step_mode: InputStepMode::Vertex,
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            attributes: &*ATTRIBUTES,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // Changed
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397057],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732911],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];