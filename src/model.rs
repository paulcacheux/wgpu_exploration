use std::{ops::Range, path::Path};

use image::ImageError;
use wgpu::util::DeviceExt;

use crate::texture::Texture;
use crate::vertex::ModelVertex;

pub struct Mesh {
    #[allow(dead_code)]
    name: String,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    pub material_index: usize,
}

pub struct Material {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    diffuse_texture: Texture,
    bind_group: wgpu::BindGroup,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn open<P: AsRef<Path>>(
        path: P,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let (obj_models, obj_materials) = tobj::load_obj(path, true)?;

        let containing_folder = path
            .parent()
            .expect("Failed to extract parent folder while loading model");

        let materials: Result<Vec<Material>, ImageError> = obj_materials
            .into_iter()
            .map(|obj_mat| -> Result<Material, ImageError> {
                let diffuse_texture = Texture::open(
                    containing_folder.join(obj_mat.diffuse_texture),
                    device,
                    queue,
                )?;

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                    ],
                    label: None,
                });

                Ok(Material {
                    name: obj_mat.name,
                    diffuse_texture,
                    bind_group,
                })
            })
            .collect();

        let materials = materials?;

        let meshes = obj_models
            .into_iter()
            .map(|model| {
                let mut vertices = Vec::new();
                for i in 0..model.mesh.positions.len() / 3 {
                    vertices.push(ModelVertex {
                        position: [
                            model.mesh.positions[i * 3],
                            model.mesh.positions[i * 3 + 1],
                            model.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            model.mesh.normals[i * 3],
                            model.mesh.normals[i * 3 + 1],
                            model.mesh.normals[i * 3 + 2],
                        ],
                    });
                }

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", path)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsage::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", path)),
                    contents: bytemuck::cast_slice(&model.mesh.indices),
                    usage: wgpu::BufferUsage::INDEX,
                });

                Mesh {
                    name: model.name,
                    vertex_buffer,
                    index_buffer,
                    index_count: model.mesh.indices.len() as u32,
                    material_index: model.mesh.material_id.unwrap_or(0),
                }
            })
            .collect();

        Ok(Model { meshes, materials })
    }
}

pub trait DrawModel<'b> {
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, uniforms: &'b wgpu::BindGroup);
    fn draw_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup);

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, uniforms: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, uniforms, 0..1);
    }

    fn draw_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, uniforms, 0..1)
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.draw_indexed(0..mesh.index_count, 0, instances);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material_index];
            self.draw_mesh_instanced(mesh, material, uniforms, instances.clone());
        }
    }
}
