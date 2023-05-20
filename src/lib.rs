use std::borrow::Cow;
use std::num::NonZeroU32;
use wgpu::{Buffer, BufferView, CommandBuffer, CommandEncoder, COPY_BYTES_PER_ROW_ALIGNMENT, FragmentState, InstanceDescriptor, VertexState};
use wgpu::util::DeviceExt;

struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }
}

struct TextureSaver;

impl TextureSaver {
    fn save_buffer_data_to_file(file_path: &str, texture_width: u32, texture_height: u32, data: Vec<u8>) {
        use image::{ImageBuffer, Rgba};
        let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(texture_width, texture_height, data).unwrap();
        image_buffer.save(file_path).unwrap();
    }
}

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
                height: 256,
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
        let (texture_width, texture_height, data) = self.map_texture_to_raw_data();
        TextureSaver::save_buffer_data_to_file(file_path, texture_width, texture_height, data);
    }

    fn map_texture_to_raw_data(&self) -> (u32, u32, Vec<u8>) {
        let uint32_size = std::mem::size_of::<u32>() as u32;
        let texture_width = self.texture.size().width;
        let texture_height = self.texture.size().height;
        let texture_size = Size::new(self.texture.size().width, self.texture.size().height);

        let texture_memory_size = texture_width * texture_height * uint32_size;
        let unpadded_bytes_per_row = texture_width * 4;
        let padded_bytes_per_row = (unpadded_bytes_per_row + COPY_BYTES_PER_ROW_ALIGNMENT - 1) / COPY_BYTES_PER_ROW_ALIGNMENT * COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.create_buffer_for_texture(texture_height, padded_bytes_per_row);
        self.copy_texture_to_buffer(texture_width, texture_height, padded_bytes_per_row, &buffer);
        let padded_data = self.wait_for_buffer_copy_completion(&buffer);
        let data = Self::read_data_from_buffer(&texture_size, texture_memory_size, padded_bytes_per_row, &padded_data);
        (texture_width, texture_height, data)
    }

    fn read_data_from_buffer(texture_size: &Size, texture_mem_size: u32, padded_bytes_per_row: u32, padded_data: &BufferView) -> Vec<u8> {
        let mut data = vec![0; (texture_mem_size * 4) as usize];
        for y in 0..texture_size.height {
            let dest_start = (y * texture_size.width * 4) as usize;
            let dest_end = ((y + 1) * texture_size.width * 4) as usize;
            let src_start = (y * padded_bytes_per_row) as usize;
            let src_end = src_start + (texture_size.width * 4) as usize;

            data[dest_start..dest_end].copy_from_slice(&padded_data[src_start..src_end]);
        }
        data
    }

    fn wait_for_buffer_copy_completion<'a>(&'a self, buffer: &'a Buffer) -> BufferView {
        let buffer_slice = buffer.slice(..);

        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        futures::executor::block_on(receiver).unwrap().unwrap();
        let padded_data = buffer_slice.get_mapped_range();
        padded_data
    }

    fn copy_texture_to_buffer(&self, texture_width: u32, texture_height: u32, padded_bytes_per_row: u32, buffer: &Buffer) {
        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        command_encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        }, wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: None,
            },
        }, wgpu::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        });

        self.queue.submit(Some(command_encoder.finish()));
    }

    fn create_buffer_for_texture(&self, texture_height: u32, padded_bytes_per_row: u32) -> Buffer {
        let buffer_size = (padded_bytes_per_row * texture_height) as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        buffer
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

    #[test]
    fn test_create_new_size() {
        let size = Size::new(100, 200);
        assert_eq!(size.width, 100);
        assert_eq!(size.height, 200);
    }
}