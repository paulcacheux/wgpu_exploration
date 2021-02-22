use cgmath::{InnerSpace, SquareMatrix, Zero};
use imgui::{im_str, Condition, Context};
use imgui_wgpu::{Renderer, RendererConfig};
use wgpu::{
    util::DeviceExt, ColorTargetState, DepthBiasState, DepthStencilState, FragmentState,
    MultisampleState, PrimitiveState, PrimitiveTopology, StencilState, VertexState,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::{
    camera::{Camera, CameraController},
    texture::Texture,
};
use crate::{
    instance::{Instance, InstanceRaw},
    model::Model,
};
use crate::{model::DrawModel, vertex::ModelVertex};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain_desc: wgpu::SwapChainDescriptor,
    swapchain: wgpu::SwapChain,
    pub window_size: PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    uniform_buffer: wgpu::Buffer,

    model: Model,

    uniforms: Uniforms,
    camera: Camera,
    camera_controller: CameraController,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    imgui_renderer: Renderer,
}

impl State {
    pub async fn new(window: &Window, imgui_context: &mut Context) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Cannot find a suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request a device and a queue");

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: false,
                        },
                        count: None,
                    },
                ],
                label: Some("Texture bind group layout"),
            });

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            front: (0.0, 0.0, -1.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: swapchain_desc.width as f32 / swapchain_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(2.0, 0.008);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform bind group layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                },
            }],
            label: Some("Uniform bind group"),
        });

        let model = Model::open(
            "res/cube/cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .expect("Failed to open model");

        let depth_texture =
            Texture::create_depth_texture(&device, &swapchain_desc, "Depth texture");

        let vs_module = device.create_shader_module(&wgpu::include_spirv!(concat!(
            env!("OUT_DIR"),
            "/shader.vert.spv"
        )));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!(concat!(
            env!("OUT_DIR"),
            "/shader.frag.spv"
        )));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: swapchain_desc.format,
                    color_blend: wgpu::BlendState::REPLACE,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
                clamp_depth: false,
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let instances = Self::build_instances();
        let raw_instances: Vec<_> = instances.iter().map(Instance::to_raw).collect();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let imgui_renderer = Renderer::new(
            imgui_context,
            &device,
            &queue,
            RendererConfig {
                texture_format: swapchain_desc.format,
                ..Default::default()
            },
        );

        State {
            surface,
            device,
            queue,
            swapchain_desc,
            swapchain,
            window_size: size,

            render_pipeline,
            uniform_bind_group,
            uniform_buffer,
            depth_texture,

            model,

            uniforms,
            camera,
            camera_controller,

            instances,
            instance_buffer,

            imgui_renderer,
        }
    }

    fn build_instances() -> Vec<Instance> {
        use cgmath::Rotation3;

        const INSTANCE_PER_ROW: usize = 10;
        const INSTANCE_COUNT: usize = INSTANCE_PER_ROW * INSTANCE_PER_ROW;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(5.0, 0.0, 5.0);

        let mut instances = Vec::with_capacity(INSTANCE_COUNT);

        for z in 0..INSTANCE_PER_ROW {
            for x in 0..INSTANCE_PER_ROW {
                let position =
                    cgmath::Vector3::new(x as f32, 0.0, z as f32) * 3.0 - INSTANCE_DISPLACEMENT;

                let rotation_axis = if position.is_zero() {
                    cgmath::Vector3::unit_z()
                } else {
                    position.normalize()
                };

                let rotation =
                    cgmath::Quaternion::from_axis_angle(rotation_axis, cgmath::Deg(45.0));

                instances.push(Instance { position, rotation })
            }
        }

        instances
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
        self.swapchain_desc.width = new_size.width;
        self.swapchain_desc.height = new_size.height;
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_desc);

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.swapchain_desc, "Depth texture");
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_window_event(event)
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) -> bool {
        self.camera_controller.process_device_event(event)
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn render(&mut self, imgui_ui: imgui::Ui) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swapchain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            for mesh in &self.model.meshes {
                let material = &self.model.materials[mesh.material_index];
                render_pass.draw_mesh_instanced(
                    mesh,
                    material,
                    &self.uniform_bind_group,
                    0..self.instances.len() as _,
                );
            }
        }

        {
            let mut imgui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Imgui render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.imgui_renderer
                .render(
                    imgui_ui.render(),
                    &self.queue,
                    &self.device,
                    &mut imgui_pass,
                )
                .expect("Failed to render UI!");
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn build_ui(&self, ui: &imgui::Ui) {
        let window = imgui::Window::new(im_str!("WGPU Exploration"));
        window
            .size([150.0, 250.0], Condition::FirstUseEver)
            .position([0.0, 0.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Camera"));
                ui.text(im_str!("Eye:"));
                ui.text(im_str!("\tx: {}", self.camera.eye.x));
                ui.text(im_str!("\ty: {}", self.camera.eye.y));
                ui.text(im_str!("\tz: {}", self.camera.eye.z));
                ui.text(im_str!("Front:"));
                ui.text(im_str!("\tx: {}", self.camera.front.x));
                ui.text(im_str!("\ty: {}", self.camera.front.y));
                ui.text(im_str!("\tz: {}", self.camera.front.z));
                ui.text(im_str!("Yaw: {:?}", self.camera_controller.yaw));
                ui.text(im_str!("Pitch: {:?}", self.camera_controller.pitch));
            });
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
