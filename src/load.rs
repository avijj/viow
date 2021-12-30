pub mod test;
pub mod vcd;

use crate::error::*;
use crate::formatting::WaveFormat;

use rug::Integer;
use std::ops::Range;

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
