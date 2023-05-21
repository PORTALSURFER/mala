use std::fmt;
use image::ImageError;

#[derive(Debug)]
pub enum TextureSaverError {
    ImageBufferCreationError,
    ImageSaveError(ImageError),
}

impl fmt::Display for TextureSaverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextureSaverError::ImageBufferCreationError => write!(f, "Failed to create image buffer"),
            TextureSaverError::ImageSaveError(error) => write!(f, "Failed to save image: {}", error),
        }
    }
}

impl From<ImageError> for TextureSaverError {
    fn from(error: ImageError) -> Self {
        TextureSaverError::ImageSaveError(error)
    }
}