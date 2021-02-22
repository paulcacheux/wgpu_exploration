use memoffset::offset_of;
use once_cell::sync::Lazy;
use wgpu::InputStepMode;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl ModelVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 3]> = Lazy::new(|| {
            [
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                    offset: offset_of!(ModelVertex, position) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    shader_location: 1,
                    offset: offset_of!(ModelVertex, tex_coords) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 2,
                    offset: offset_of!(ModelVertex, normal) as _,
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
