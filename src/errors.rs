use failure::{Backtrace, Context, Fail};
use image::ImageError;
use std::fmt;
use std::fmt::Display;
use std::io::Error as IOError;
use std::num::ParseFloatError;

#[derive(Debug)]
pub struct PixelMatchError {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Is not number")]
    ParseFloatError,
    #[fail(display = "Failed to open image file")]
    InvalidImageFile,
    #[fail(display = "Image sizes does not match")]
    SizeUnmatch,
    #[fail(display = "Failed to save diff image")]
    IOError,
}

impl Fail for PixelMatchError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for PixelMatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl PixelMatchError {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for PixelMatchError {
    fn from(kind: ErrorKind) -> PixelMatchError {
        PixelMatchError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for PixelMatchError {
    fn from(inner: Context<ErrorKind>) -> PixelMatchError {
        PixelMatchError { inner }
    }
}

impl From<ParseFloatError> for PixelMatchError {
    fn from(error: ParseFloatError) -> PixelMatchError {
        PixelMatchError {
            inner: error.context(ErrorKind::ParseFloatError),
        }
    }
}

impl From<ImageError> for PixelMatchError {
    fn from(error: ImageError) -> PixelMatchError {
        PixelMatchError {
            inner: error.context(ErrorKind::InvalidImageFile),
        }
    }
}

impl From<IOError> for PixelMatchError {
    fn from(error: IOError) -> PixelMatchError {
        PixelMatchError {
            inner: error.context(ErrorKind::IOError),
        }
    }
}
