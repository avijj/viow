pub mod test;
pub mod vcd;

use crate::formatting::WaveFormat;

use rug::Integer;
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


#[derive(Clone)]
pub struct SignalDeclaration {
    pub name: String,
    pub format: WaveFormat,
}

pub trait LoadDeclarations {
    fn load_declarations(&self) -> Result<Vec<SignalDeclaration>>;
}

pub trait LoadLength {
    fn load_length(&self) -> Result<usize>;
}

pub trait LoadWaveform {
    fn load_waveform(&self, name: impl AsRef<str>, cycles: Range<usize>) -> Result<Vec<Integer>>;
}
