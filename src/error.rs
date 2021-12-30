use std::ops::Range;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum Error {
    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("Named signal '{0:}' not found")]
    NotFound(String),

    #[error("The given range {0:?} is invalid within limits of {1:?}.")]
    InvalidRange(Range<usize>, Range<usize>),

    #[error("The given text '{0:}' can not be interpreted as time.")]
    InvalidTime(String),
}

pub type Result<T> = std::result::Result<T, Error>;

