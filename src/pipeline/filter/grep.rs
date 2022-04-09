use regex::Regex;
use rug;

use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;

pub struct Grep {
    re: Regex,
}

impl Grep {
    pub fn new(expression: &str) -> Result<Self> {
        let re = Regex::new(expression)?;

        Ok(Self {
            re
        })
    }
}


impl<I> TranslateSignals<I> for Grep {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        let filtered_signals = signals.into_iter()
            .filter(|signal| {
                if signal.format == WaveFormat::Comment {
                    true
                } else {
                    self.re.is_match(&signal.name)
                }
            })
            .collect();

        Ok(filtered_signals)
    }

    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter> {
        Ok(signals)
    }
}


impl Transform for Grep {
    type Value = rug::Integer;
}

impl ConfigurePipeline for Grep {}

impl<I> Filter<I, rug::Integer> for Grep {}
