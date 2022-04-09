use rug;

use crate::error::*;
use crate::data::*;

pub struct ReplacePrefix {
    prefix: String,
    replacement: String,
}

impl ReplacePrefix {
    pub fn new(prefix: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
           prefix: prefix.into(),
           replacement: replacement.into(), 
        }
    }
}


impl<I> TranslateSignals<I> for ReplacePrefix {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        let filtered_signals = signals.into_iter()
            .map(|mut signal| {
                if signal.name.starts_with(&self.prefix) {
                    signal.name = signal.name.replace(&self.prefix, &self.replacement);
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


impl Transform for ReplacePrefix {
    type Value = rug::Integer;
}

impl ConfigurePipeline for ReplacePrefix {}

impl<I> Filter<I, rug::Integer> for ReplacePrefix {}

