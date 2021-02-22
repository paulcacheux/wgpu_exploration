use once_cell::sync::Lazy;
use wgpu::InputStepMode;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 4]> = Lazy::new(|| {
            [
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 5,
                    offset: (std::mem::size_of::<[f32; 4]>() * 0) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 6,
                    offset: (std::mem::size_of::<[f32; 4]>() * 1) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 7,
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as _,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 8,
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as _,
                },
            ]
        });

        wgpu::VertexBufferLayout {
            step_mode: InputStepMode::Instance,
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            attributes: &*ATTRIBUTES,
        }
    }
}
