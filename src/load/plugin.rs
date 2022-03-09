use crate::error::*;
use crate::data::*;
use crate::formatting::*;
use viow_plugin_api::{
    WaveLoadType,
    FiletypeLoader_Ref,
};
use abi_stable::std_types::*;
use rug::{
    Integer,
    integer::Order
};
use ndarray::prelude::*;

pub struct PluggedLoader {
    plugin: FiletypeLoader_Ref,
    loader: WaveLoadType,
    signals: Vec<String>,
    cycle_time: SimTime,
    num_cycles: usize,
}

impl PluggedLoader {
    pub fn new(plugin: FiletypeLoader_Ref, input: impl Into<RString>, cycle_time: SimTime) -> Result<Self> {
        let cycle_time_ps = cycle_time.as_ps()
            .ok_or(Error::Internal(format!("Cycle time {cycle_time:?} to large to represent in units of ps")))?;
        let mut loader = plugin.open()(&input.into(), cycle_time_ps).into_result()?;

        let signals = loader.list_signal_names().into_result()?
            .into_iter()
            .map(|x| x.into_string())
            .collect();

        let num_cycles = loader.count_cycles().into_result()? as usize;

        Ok(Self {
            plugin,
            loader,
            signals,
            cycle_time,
            num_cycles,
        })
    }
}


impl QuerySource for PluggedLoader {
    type Id = String;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let rv: Self::IntoSignalIter = self
            .signals
            .iter()
            .map(|txt| Signal {
                id: txt.clone(),
                name: txt.clone(),
                format: WaveFormat::Bit   // TODO determine from plugin
            })
            .collect();

        Ok(rv)
    }

    fn query_time_range(&self) -> Result<SimTimeRange> {
        let start = SimTime::zero();
        let stop = self.cycle_time * (self.num_cycles as u64);

        Ok(SimTimeRange(start, stop))
    }

    fn query_time(&self, cycle: usize) -> SimTime {
        self.cycle_time * (cycle as u64)
    }

    fn query_cycle_count(&self) -> usize {
        self.num_cycles
    }
}

impl LookupId for PluggedLoader {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: &Self::FromId) -> Result<Self::ToId> {
        let pos = self.signals.iter().position(|x| *x == *id);

        match pos {
            Some(p) => Ok(p),
            None => Err(Error::NotFound(id.clone())),
        }
    }

    fn rev_lookup_id(&self, id: &Self::ToId) -> Result<Self::FromId> {
        if *id < self.signals.len() {
            Ok(self.signals[*id].clone())
        } else {
            Err(Error::IdOutOfRange(*id, 0..self.signals.len()))
        }
    }
}

impl Sample for PluggedLoader {
    type Id = String;
    type Value = Integer;

    fn sample(
        &self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        let start_cycle = times.0 / self.cycle_time;
        let stop_cycle = times.1 / self.cycle_time;

        // load subset
        let rids: RVec<_> = ids.iter()
            .map(|x| RString::from(x.as_str()))
            .collect();
        let subset = self.loader.load(&rids, Tuple2::from((start_cycle, stop_cycle)))
            .into_result()?;

        // convert to Integer
        let num_cycles = (stop_cycle - start_cycle) as usize;
        let num_signals = ids.len();
        let mut data = Array2::default((num_cycles, num_signals));

        for (row_i, mut row) in data.outer_iter_mut().enumerate() {
            for (col_i, _) in ids.iter().enumerate() {
                let bits = self.loader.extract(&subset, col_i as u64, row_i as u64)
                    .into_vec();
                row[[col_i]] = Integer::from_digits(&bits, Order::Msf);
            }
        }

        Ok(data)
    }
}

impl Source<String, usize, Integer> for PluggedLoader {}


