use crate::data::*;
use crate::error::*;
use crate::formatting::WaveFormat;

use std::collections::HashMap;

pub struct SignalList {
    signals: HashMap<String, usize>,
    enabled: bool,
}

impl SignalList {
    pub fn new(signals: impl IntoIterator<Item = String>) -> Self {
        let mut hash_map = HashMap::new();

        for (i, signal) in signals.into_iter().enumerate() {
            hash_map.insert(signal, i);
        }

        Self { signals: hash_map, enabled: true }
    }
}

impl<I> TranslateSignals<I> for SignalList {
    type IntoSigIter = Vec<Signal<I>>;
    type IntoIdIter = Vec<I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter> {
        if !self.enabled {
            return Ok(signals);
        }

        let mut filtered_signals: Vec<_> = signals
            .into_iter()
            // Assign sort keys to all elements
            .scan(0, |state, x| {
                if let Some(sort_key) = self.signals.get(&x.name) {
                    *state = *sort_key;
                    Some((*sort_key, x, true))
                } else {
                    Some((*state, x, false))
                }
            })
            // Remove signals not in list
            .filter_map(|(sort_key, signal, found)| {
                if found || signal.format == WaveFormat::Comment {
                    Some((sort_key, signal))
                } else {
                    None
                }
            })
            .collect();

        // sort by sort key given in list (stable sort)
        filtered_signals.sort_by_key(|signal| signal.0);

        // Convert to expected format without sort key
        let sorted_signals = filtered_signals
            .into_iter()
            .map(|(_, signal)| signal)
            .collect();

        Ok(sorted_signals)
    }

    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter> {
        Ok(signals)
    }
}

impl Transform for SignalList {
    type Value = rug::Integer;
}

impl ConfigurePipeline for SignalList {
    fn configure_pipeline(&mut self, config: &PipelineConfig) -> Result<()> {
        self.signals.clear();

        for (i, signal) in config.name_list.iter().enumerate() {
            self.signals.insert(signal.clone(), i);
        }

        self.enabled = config.enable_filter_list;

        Ok(())
    }
}

impl<I> Filter<I, rug::Integer> for SignalList {}
