use std::fmt;
use image::ImageError;
use wgpu::{BufferAsyncError, RequestDeviceError};
use crate::texture_saver::TextureSaverError;

#[derive(Debug)]
pub enum RendererError {
    BufferMapError(BufferAsyncError),
    TextureSaveFailure(TextureSaverError),
    NoAdapterFound,
    DeviceRequestError(RequestDeviceError),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RendererError::BufferMapError(error) => write!(f, "Failed to map buffer: {}", error),
            RendererError::TextureSaveFailure(error) => write!(f, "Failed to save texture to file: {}", error),
            RendererError::NoAdapterFound => write!(f, "No adapter found"),
            RendererError::DeviceRequestError(error) => write!(f, "Failed to request device: {}", error),
        }
    }
}

impl From<BufferAsyncError> for RendererError {
    fn from(error: BufferAsyncError) -> Self {
        RendererError::BufferMapError(error)
    }
}

impl From<TextureSaverError> for RendererError {
    fn from(error: TextureSaverError) -> Self {
        RendererError::TextureSaveFailure(error)
    }
}

