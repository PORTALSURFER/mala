mod error;

pub use error::TextureSaverError;

use image::{ImageBuffer, Rgba};
use crate::size::Size;

pub struct TextureSaver;

impl TextureSaver {
    pub(crate) fn save_buffer_data_to_file(file_path: &str, texture_size: &Size, data: Vec<u8>) -> Result<(), TextureSaverError> {
        let image_buffer = Self::create_image_buffer(texture_size, data)?;
        image_buffer.save(file_path).map_err(TextureSaverError::from)
    }

    fn create_image_buffer(texture_size: &Size, data: Vec<u8>) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, TextureSaverError> {
        ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size.width, texture_size.height, data).ok_or(TextureSaverError::ImageBufferCreationError)
    }
}