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
}

pub type Result<T> = std::result::Result<T, Error>;

