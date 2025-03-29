use std::sync::Arc;

use anyhow::{Context, Result};
use cgmath::{Deg, Matrix2, Vector2};
use palette::Srgba;
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct Renderer {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    current_index: u16,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
            .await
            .context("cannot create adapter from wgpu instance")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
                None, // Trace path
            )
            .await?;

        let surface = instance.create_surface(window.clone())?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0].add_srgb_suffix();

        let size = window.inner_size();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }];

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 1024 * std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: 1024 * std::mem::size_of::<u16>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let renderer = Self {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,

            render_pipeline,
            vertex_buffer,
            index_buffer,

            vertices: Vec::new(),
            indices: Vec::new(),
            current_index: 0, // the current vertex index. Will be used to create indicies
        };

        renderer.configure_surface();

        Ok(renderer)
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn begin_drawing(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.current_index = 0;
    }

    pub fn end_drawing(&mut self) -> Result<()> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let clear_color = Srgba::new(67, 140, 127, 1).into_linear();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color.red,
                        g: clear_color.green,
                        b: clear_color.blue,
                        a: clear_color.alpha,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Update Drawing Data with vertices & indices:
        if self.indices.len() % 2 != 0 {
            // pad indicies to align with u16
            self.indices.push(0)
        }
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        // Drawing:
        if !self.indices.is_empty() {
            // Render
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }

        // End the renderpass.
        drop(render_pass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }

    fn to_ndc(&self, pos: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(
            2.0 * pos.x / self.size.width as f32 - 1.0,
            -(2.0 * pos.y / self.size.height as f32 - 1.0),
        )
    }

    pub fn draw_rectangle(
        &mut self,
        pos: Vector2<f32>,
        width: f32,
        height: f32,
        color: Srgba,
        rotation: Deg<f32>,
    ) {
        // Define corners in local space (relative to center)
        let origin = Vector2::new(pos.x + width / 2.0, pos.y + height / 2.0);
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let local_top_left = Vector2::new(-half_width, -half_height);
        let local_top_right = Vector2::new(half_width, -half_height);
        let local_bottom_right = Vector2::new(half_width, half_height);
        let local_bottom_left = Vector2::new(-half_width, half_height);

        // Apply rotation and translate back to world space
        let rotation_matrix = Matrix2::from_angle(rotation);
        let rotated_top_left = rotation_matrix * local_top_left + origin;
        let rotated_top_right = rotation_matrix * local_top_right + origin;
        let rotated_bottom_right = rotation_matrix * local_bottom_right + origin;
        let rotated_bottom_left = rotation_matrix * local_bottom_left + origin;

        // Create Rectangle (Vertices):
        self.vertices.push(Vertex {
            position: self.to_ndc(rotated_top_left).into(),
            color: color.into(),
        });
        self.vertices.push(Vertex {
            position: self.to_ndc(rotated_top_right).into(),
            color: color.into(),
        });
        self.vertices.push(Vertex {
            position: self.to_ndc(rotated_bottom_right).into(),
            color: color.into(),
        });
        self.vertices.push(Vertex {
            position: self.to_ndc(rotated_bottom_left).into(),
            color: color.into(),
        });

        // Create Rectangle CCW (Indices)
        self.indices.push(self.current_index + 2);
        self.indices.push(self.current_index + 1);
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 3);
        self.indices.push(self.current_index + 2);
        self.indices.push(self.current_index);

        self.current_index += 4;
    }

    pub fn draw_triangle(
        &mut self,
        v1: Vector2<f32>,
        v2: Vector2<f32>,
        v3: Vector2<f32>,
        color: Srgba,
        rotation: Deg<f32>,
    ) {
        let origin = Vector2::new((v1.x + v2.x + v3.x) / 3.0, (v1.y + v2.y + v3.y) / 3.0);

        // Translate to origin
        let local_v1 = v1 - origin;
        let local_v2 = v2 - origin;
        let local_v3 = v3 - origin;

        let rotation_matrix = Matrix2::from_angle(rotation);
        let r1 = rotation_matrix * local_v1 + origin;
        let r2 = rotation_matrix * local_v2 + origin;
        let r3 = rotation_matrix * local_v3 + origin;

        self.vertices.push(Vertex {
            position: self.to_ndc(r1).into(),
            color: color.into(),
        });
        self.vertices.push(Vertex {
            position: self.to_ndc(r2).into(),
            color: color.into(),
        });
        self.vertices.push(Vertex {
            position: self.to_ndc(r3).into(),
            color: color.into(),
        });

        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);
        self.indices.push(self.current_index + 2);

        self.current_index += 3;
    }
}
