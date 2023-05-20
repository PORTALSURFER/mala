use std::borrow::Cow;
use std::num::NonZeroU32;
use wgpu::{CommandBuffer, CommandEncoder, FragmentState, InstanceDescriptor, VertexState};
use wgpu::util::DeviceExt;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    is_initialized: bool,
}

impl Renderer {
    pub fn new_blocking() -> Self {
        futures::executor::block_on(Self::new())
    }

    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        }, None).await.unwrap();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 800,
                height: 600,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            device,
            queue,
            texture,
            view,
            is_initialized: true,
        }
    }

    pub fn render_triangle(&self) {
        let vertices = [
            [-0.5, -0.5],   // Bottom-left vertex
            [0.5, -0.5],   // Bottom-right vertex
            [0.0, 0.5],   // Top vertex
        ];

        let indices = [0, 1, 2];

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader.wgsl"))),
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });


        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        let command_buffer = command_encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }

    pub fn save_to_texture_on_disk(&self, file_path: &str) {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (800 * 600 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = 800 * 4;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        command_encoder.copy_texture_to_buffer(wgpu::ImageCopyTextureBase {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        }, wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: None,
            },
        }, wgpu::Extent3d {
            width: 800,
            height: 600,
            depth_or_array_layers: 1,
        });


        let (sender, receiver) = futures::channel::oneshot::channel();
        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.queue.on_submitted_work_done(move || {
            sender.send(()).expect("TODO: panic message");
        });

        futures::executor::block_on(receiver).unwrap();


        let (sender, receiver) = futures::channel::oneshot::channel();
        let mapping = buffer.slice(..).map_async(wgpu::MapMode::Read, move |e| { sender.send(()).expect("TODO: panic message"); });
        futures::executor::block_on(receiver).unwrap();
        self.device.poll(wgpu::Maintain::Wait);

        let data = buffer.slice(..).get_mapped_range();
        std::fs::write(file_path, &data).unwrap();
        buffer.unmap();
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_initialization() {
        let renderer = Renderer::new_blocking();
        assert!(renderer.is_initialized(), "Renderer was not initialized");
    }

    #[test]
    fn test_render_to_texture_on_disk() {
        let renderer = Renderer::new_blocking();
        renderer.render_triangle();
        renderer.save_to_texture_on_disk("output.png");
        assert!(std::path::Path::new("output.png").exists(), "Texture was not saved on disk");
    }
}