pub mod test;

use crate::formatting::WaveFormat;

use rug::Integer;
use std::ops::Range;

pub struct SignalDeclaration {
    pub name: String,
    pub format: WaveFormat,
}

pub trait LoadDeclarations {
    fn load_declarations(&self) -> Vec<SignalDeclaration>;
}

pub trait LoadLength {
    fn load_length(&self) -> usize;
}

pub trait LoadWaveform {
    fn load_waveform(&self, name: impl AsRef<str>, cycles: Range<usize>) -> Vec<Integer>;
}
