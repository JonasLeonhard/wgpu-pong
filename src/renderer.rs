use std::sync::Arc;

use anyhow::{Context, Result};
use palette::{Srgba, encoding::linear};
use winit::window::Window;

struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct Renderer {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,

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
        let surface_format = cap.formats[0];

        let size = window.inner_size();

        let renderer = Self {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,

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

    pub fn end_drawing(&self) -> Result<()> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let clear_color = Srgba::new(67, 140, 127, 1).into_linear();

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        // Drawing:
        if !self.indices.is_empty() {
            // TODO!:
            // render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }

        // End the renderpass.
        drop(render_pass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }

    pub fn draw_rectangle(
        &mut self,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
        color: Srgba,
    ) {
        // Normalized Coordinates
        let x1 = 2.0 * pos_x as f32 / self.size.width as f32 - 1.0;
        let y1 = -(2.0 * pos_y as f32 / self.size.height as f32 - 1.0); // Flip Y axis
        let x2 = 2.0 * (pos_x + width) as f32 / self.size.width as f32 - 1.0;
        let y2 = -(2.0 * (pos_y + height) as f32 / self.size.height as f32 - 1.0);

        let linear_color = color.into_linear();

        // Create Rectangle (Verticies):
        // Top-left
        self.vertices.push(Vertex {
            position: [x1, y1],
            color: linear_color.into(),
        });
        // Top-right
        self.vertices.push(Vertex {
            position: [x2, y1],
            color: linear_color.into(),
        });
        // Bottom-right
        self.vertices.push(Vertex {
            position: [x2, y2],
            color: linear_color.into(),
        });
        // Bottom-left
        self.vertices.push(Vertex {
            position: [x1, y2],
            color: linear_color.into(),
        });

        // Create Rectangle (Indicies)
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);
        self.indices.push(self.current_index + 2);
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 2);
        self.indices.push(self.current_index + 3);

        self.current_index += 4;
    }
}
