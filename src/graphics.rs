use anyhow::{bail, Result};
use bytemuck::Zeroable;
use glam::Vec2;
use std::sync::Arc;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device, DeviceDescriptor,
    Features, FragmentState, Instance, InstanceDescriptor, Limits, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, Surface, SurfaceConfiguration,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::Event, window::Window};

use crate::{
    camera::{Camera, CameraController},
    vertex::Vertex,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    // _padding: [u8; 8],
    // aspect: f32,
    transform: glam::Mat4,
    color: glam::Vec4,
}

pub struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Arc<Window>,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_bind_group: wgpu::BindGroup,
    line_pipeline: RenderPipeline,
    point_pipeline: RenderPipeline,
    point_buffer: wgpu::Buffer,
    point_count: u32,
}

impl Graphics {
    pub async fn new(window: Window, vertex_data: &[Vertex], indices: &[u32]) -> Result<Self> {
        let window = Arc::new(window);

        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await;
        let Some(adapter) = adapter else {
            bail!("No adapter found");
        };
        let config = find_config(&surface, &adapter, size);
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits {
                        max_buffer_size: 786_432_000,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .await?;
        surface.configure(&device, &config);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let max_highlighted = vertex_data.len() as u64;
        let point_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: max_highlighted,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let num_indices = indices.len() as u32;

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[Camera::zeroed().matrix()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
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
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let line_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let line_pipeline =
            make_pipeline(&device, &config, &line_layout, PrimitiveTopology::LineStrip);
        let point_pipeline =
            make_pipeline(&device, &config, &line_layout, PrimitiveTopology::PointList);

        Ok(Self {
            device,
            queue,
            surface,
            window,
            line_pipeline,
            point_pipeline,
            point_buffer,
            config,
            vertex_buffer,
            uniform_buffer,
            index_buffer,
            num_indices,
            uniform_bind_group,
            size,
            point_count: 0,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn size_vec(&self) -> Vec2 {
        Vec2::new(self.size.width as f32, self.size.height as f32)
    }

    pub fn update(&mut self, uniforms: &CameraController, vertices: &[Vertex]) {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms.matrix()]),
        );
        let v_bytes = (vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        if v_bytes > self.point_buffer.size() {
            println!("Resizing point buffer");
            self.point_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: v_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.queue
            .write_buffer(&self.point_buffer, 0, bytemuck::cast_slice(vertices));
        self.point_count = vertices.len() as u32;
    }

    pub fn render(&self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.line_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // render_pass.set_pipeline(&self.point_pipeline);
            // render_pass.set_vertex_buffer(0, self.point_buffer.slice(..));
            // render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            // render_pass.draw(0..self.point_count, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
    }

    pub fn input(&self, event: &Event<()>) -> bool {
        false
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

fn find_config(
    surface: &Surface,
    adapter: &wgpu::Adapter,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    let surface_config = surface.get_capabilities(adapter);
    let format = surface_config
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .unwrap_or(&surface_config.formats[0]);

    SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_config.alpha_modes[0],
        view_formats: vec![],
    }
}

fn make_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    layout: &PipelineLayout,
    topology: PrimitiveTopology,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
    });
    let vertex = wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    };
    let fragment = FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex,
        fragment: Some(fragment),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: if topology == wgpu::PrimitiveTopology::LineStrip {
                Some(wgpu::IndexFormat::Uint32)
            } else {
                None
            },
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
            unclipped_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
