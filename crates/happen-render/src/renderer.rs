use std::sync::Arc;
use wgpu::util::DeviceExt;
use happen_math::Transform;

use crate::camera::{Camera, CameraUniform};
use crate::material::MaterialAssets;
use crate::mesh::MeshAssets;
use crate::vertex::Vertex;
use crate::MeshRenderer;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window: Arc<winit::window::Window>,
}

impl GpuContext {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Happen GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

pub struct RenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub model_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl RenderState {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Mesh Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
            });

        let camera_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let model_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Model Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let material_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Material Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_bind_group_layout,
                        &model_bind_group_layout,
                        &material_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Happen Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let camera_uniform = CameraUniform::new();
        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct ModelUniform {
            model: [[f32; 4]; 4],
            normal_matrix: [[f32; 4]; 4],
        }

        let model_uniform = ModelUniform {
            model: happen_math::Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: happen_math::Mat4::IDENTITY.to_cols_array_2d(),
        };

        let model_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let model_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        });

        let (depth_texture, depth_view) = create_depth_texture(
            &gpu.device,
            gpu.surface_config.width,
            gpu.surface_config.height,
        );

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            model_buffer,
            model_bind_group,
            material_bind_group_layout,
            depth_texture,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (tex, view) = create_depth_texture(device, width, height);
        self.depth_texture = tex;
        self.depth_view = view;
    }

    pub fn render(
        &self,
        gpu: &GpuContext,
        world: &happen_core::World,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = gpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut camera_uniform = CameraUniform::new();
        let mut clear_color = happen_math::Color::CORNFLOWER_BLUE;

        let entities = world.all_entities();
        for &entity in &entities {
            if let (Some(cam), Some(transform)) = (
                world.get_component::<Camera>(entity),
                world.get_component::<Transform>(entity),
            ) {
                if cam.active {
                    let view_matrix = Camera::view_matrix(transform.position, transform.rotation);
                    let proj_matrix = cam.projection.matrix();
                    camera_uniform.update(view_matrix, proj_matrix, transform.position);
                    clear_color = cam.clear_color;
                    break;
                }
            }
        }

        gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Happen Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Happen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            let mesh_assets = world.get_resource::<MeshAssets>();
            let material_assets = world.get_resource::<MaterialAssets>();

            if let (Some(meshes), Some(materials)) = (mesh_assets, material_assets) {
                for &entity in &entities {
                    let (Some(mesh_renderer), Some(transform)) = (
                        world.get_component::<MeshRenderer>(entity),
                        world.get_component::<Transform>(entity),
                    ) else {
                        continue;
                    };

                    if !mesh_renderer.visible {
                        continue;
                    }

                    let Some(gpu_mesh) = meshes.get(mesh_renderer.mesh) else {
                        continue;
                    };
                    let Some(gpu_material) = materials.get(mesh_renderer.material) else {
                        continue;
                    };

                    #[repr(C)]
                    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
                    struct ModelData {
                        model: [[f32; 4]; 4],
                        normal_matrix: [[f32; 4]; 4],
                    }

                    let model_data = ModelData {
                        model: transform.to_matrix().to_cols_array_2d(),
                        normal_matrix: transform.normal_matrix().to_cols_array_2d(),
                    };

                    gpu.queue.write_buffer(
                        &self.model_buffer,
                        0,
                        bytemuck::cast_slice(&[model_data]),
                    );

                    render_pass.set_bind_group(1, &self.model_bind_group, &[]);
                    render_pass.set_bind_group(2, &gpu_material.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        gpu_mesh.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

