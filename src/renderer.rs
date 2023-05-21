mod tests;

use wgpu::{Adapter, Buffer, BufferView, CommandEncoder, COPY_BYTES_PER_ROW_ALIGNMENT, Device, Face, FragmentState, Instance, InstanceDescriptor, PrimitiveState, Queue, RenderPipeline, RequestDeviceError, ShaderModule, Texture, TextureView, VertexState};
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use crate::constants::{RENDER_TARGET_HEIGHT, RENDER_TARGET_WIDTH, VertexFloat};
use crate::error::RendererError;
use crate::size::Size;
use crate::texture_saver::TextureSaver;
use crate::vertex::Vertex;

async fn request_adapter(instance: Instance) -> Option<Adapter> {
    instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }).await
}

fn create_instance() -> Instance {
    Instance::new(InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: Default::default(),
    })
}

async fn request_device(adapter: Adapter) -> Result<(Device, Queue), RequestDeviceError> {
    adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::default(),
    }, None).await
}

fn create_render_texture(device: &Device) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: RENDER_TARGET_WIDTH,
            height: RENDER_TARGET_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    texture: Texture,
    view: TextureView,
    is_initialized: bool,
}

impl Renderer {
    pub fn new_blocking() -> Result<Self, RendererError> {
        futures::executor::block_on(Self::new())
    }

    pub async fn new() -> Result<Self, RendererError> {
        let instance = create_instance();
        let adapter = request_adapter(instance).await.ok_or(RendererError::NoAdapterFound)?;
        let (device, queue) = request_device(adapter).await.map_err(RendererError::DeviceRequestError)?;
        let texture = create_render_texture(&device);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            device,
            queue,
            texture,
            view,
            is_initialized: true,
        })
    }

    pub fn render_triangle(&self) -> Result<(), ()> {
        let triangle_bottom_left_vertex = Vertex::new(-0.5, -0.5);
        let triangle_bottom_right_vertex = Vertex::new(0.5, -0.5);
        let triangle_top_vertex = Vertex::new(0.0, 0.5);
        let vertices = [triangle_bottom_left_vertex.position, triangle_bottom_right_vertex.position, triangle_top_vertex.position];
        let indices = [0, 1, 2];

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        let shader = self.create_default_shader();
        let pipeline = self.create_default_pipeline(&shader)?;
        let vertex_buffer = self.create_vertex_buffer(&vertices);
        let index_buffer = self.create_index_buffer(&indices);

        self.triangle_render_pass(indices, &mut command_encoder, &pipeline, vertex_buffer, index_buffer);

        let command_buffer = command_encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
        Ok(())
    }

    fn triangle_render_pass(&self, indices: [i32; 3], command_encoder: &mut CommandEncoder, pipeline: &RenderPipeline, vertex_buffer: Buffer, index_buffer: Buffer) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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

    fn create_index_buffer(&self, indices: &[i32; 3]) -> Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    fn create_vertex_buffer(&self, vertices: &[[VertexFloat; 2]; 3]) -> Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn create_default_pipeline(&self, shader: &ShaderModule) -> Result<RenderPipeline, ()> {
        Ok(self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
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
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        }))
    }

    pub fn create_default_shader(&self) -> ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader.wgsl"))),
        })
    }

    pub fn save_to_texture_on_disk(&self, file_path: &str) -> Result<(), RendererError> {
        let (texture_size, data) = self.map_texture_to_raw_data()?;
        TextureSaver::save_buffer_data_to_file(file_path, &texture_size, data)?;
        Ok(())
    }

    fn map_texture_to_raw_data(&self) -> Result<(Size, Vec<u8>), RendererError> {
        let texture_size = Size::new(self.texture.size().width, self.texture.size().height);
        let unpadded_bytes_per_row = texture_size.width * 4;
        let padded_bytes_per_row = (unpadded_bytes_per_row + COPY_BYTES_PER_ROW_ALIGNMENT - 1) / COPY_BYTES_PER_ROW_ALIGNMENT * COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.create_buffer_for_texture(texture_size.height, padded_bytes_per_row);
        self.copy_texture_to_buffer(&texture_size, padded_bytes_per_row, &buffer);
        let padded_data = self.wait_for_buffer_copy_completion(&buffer)?;
        let texture_data = Self::read_data_from_buffer(&texture_size, padded_bytes_per_row, &padded_data);
        Ok((texture_size, texture_data))
    }

    fn get_texture_memory_size(texture_size: &Size) -> u32 {
        let uint32_size = std::mem::size_of::<u32>() as u32;
        texture_size.get_area() * uint32_size
    }

    fn read_data_from_buffer(texture_size: &Size, padded_bytes_per_row: u32, buffer_view: &BufferView) -> Vec<u8> {
        let texture_memory_size = Self::get_texture_memory_size(&texture_size);
        let mut texture_data = vec![0; (texture_memory_size * 4) as usize];
        for y in 0..texture_size.height {
            let dest_start = (y * texture_size.width * 4) as usize;
            let dest_end = ((y + 1) * texture_size.width * 4) as usize;
            let src_start = (y * padded_bytes_per_row) as usize;
            let src_end = src_start + (texture_size.width * 4) as usize;
            texture_data[dest_start..dest_end].copy_from_slice(&buffer_view[src_start..src_end]);
        }
        texture_data
    }

    fn wait_for_buffer_copy_completion<'a>(&self, buffer: &'a Buffer) -> Result<BufferView<'a>, RendererError> {
        let buffer_slice = buffer.slice(..);

        futures::executor::block_on(async {
            buffer_slice.map_async(wgpu::MapMode::Read, |result| {
                if let Err(error) = result {
                    eprintln!("Failed to map buffer: {:?}", error);
                    panic!("Failed to map buffer: {:?}", error);
                }
            });

            self.device.poll(wgpu::Maintain::Wait);

            let padded_data = buffer_slice.get_mapped_range();
            Ok(padded_data)
        })
    }

    fn copy_texture_to_buffer(&self, texture_size: &Size, padded_bytes_per_row: u32, buffer: &Buffer) {
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
            width: texture_size.width,
            height: texture_size.height,
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
