use regex::RegexSet;

use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;


pub struct Ignore {
    allow: RegexSet,
    deny: RegexSet,
}


impl Ignore {
    pub fn new<T: AsRef<str>>(allow: &[T], deny: &[T]) -> Result<Self> {
        let allow_re = RegexSet::new(allow)?;
        let deny_re = RegexSet::new(deny)?;

        Ok(Self {
            allow: allow_re,
            deny: deny_re,
        })
    }
}


impl<I> TranslateSignals<I> for Ignore {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        let filtered_signals = signals.into_iter()
            .filter(|signal| {
                if signal.format == WaveFormat::Comment {
                    true
                } else if self.allow.is_match(&signal.name) {
                    true
                } else if self.deny.is_match(&signal.name) {
                    false
                } else {
                    true
                }
            })
            .collect();

        Ok(filtered_signals)
    }

    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter> {
        Ok(signals)
    }
}

impl Transform for Ignore {
    type Value = rug::Integer;

    fn transform(&self, _: &mut Self::Value) {
    }
}

impl ConfigurePipeline for Ignore {}

impl<I> Filter<I, rug::Integer> for Ignore {}
