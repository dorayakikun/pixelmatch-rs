use image::ImageError;
use std::io::Error as IOError;
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Is not number")]
    ParseFloatError,
    #[error("Failed to open image file. caused by {0}")]
    InvalidImageFile(String),
    #[error("Image sizes does not match. before: {before:?} after: {after:?}")]
    SizeUnmatch {
        before: (u32, u32),
        after: (u32, u32),
    },
    #[error("Failed to save diff image")]
    IoError,
}

impl From<ParseFloatError> for Error {
    fn from(_error: ParseFloatError) -> Self {
        Error::ParseFloatError
    }
}

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Error::InvalidImageFile(error.to_string())
    }
}

impl From<IOError> for Error {
    fn from(_error: IOError) -> Self {
        Error::IoError
    }
}
