use regex::RegexSet;

use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;


pub struct Analog {
    patterns: RegexSet,
    min: f64,
    max: f64,
}


impl Analog {
    pub fn new<T: AsRef<str>>(patterns: &[T], min: f64, max: f64) -> Result<Self> {
        let patterns = RegexSet::new(patterns)?;

        Ok(Self {
            patterns,
            min,
            max,
        })
    }
}


impl<I> TranslateSignals<I> for Analog {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        let filtered_signals = signals.into_iter()
            .map(|mut signal| {
                if self.patterns.is_match(&signal.name) {
                    signal.format = match signal.format {
                        WaveFormat::BitVector(sz)
                        | WaveFormat::Vector(sz) => {
                            WaveFormat::Analog(sz, self.min, self.max)
                        }
                        _ => signal.format
                    };
                }

                signal
            })
            .collect();

        Ok(filtered_signals)
    }

    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter> {
        Ok(signals)
    }
}

impl Transform for Analog {
    type Value = rug::Integer;
}

impl ConfigurePipeline for Analog {}

impl<I> Filter<I, rug::Integer> for Analog {}

