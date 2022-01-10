use rug;

use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;

pub struct RemoveComments { }

impl RemoveComments {
    pub fn new() -> Self {
        Self {}
    }
}


impl<I> TranslateSignals<I> for RemoveComments {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        let filtered_signals = signals.into_iter()
            .filter(|signal| signal.format != WaveFormat::Comment)
            .collect();

        Ok(filtered_signals)
    }

    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter> {
        Ok(signals)
    }
}


impl Transform for RemoveComments {
    type Value = rug::Integer;

    fn transform(&self, _: &mut Self::Value) {
    }
}

impl ConfigurePipeline for RemoveComments {}

impl<I> Filter<I, rug::Integer> for RemoveComments {}
