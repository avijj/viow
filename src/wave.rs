mod cache;
mod pipeline_cid;

use cache::*;
use pipeline_cid::PipelineCId;
use crate::error::*;
use crate::formatting::{WaveFormat,format_value};
use crate::data::*;
use crate::pipeline::*;
use crate::config::Config;

use ndarray::prelude::*;
use ndarray;
use rug::Integer;

const SEARCH_HORIZON: usize = 1024;

// Do not sample data on load or ever hold any data in Wave. Always defer to pipe to fetch
// data:
//  - ✓ replace slice_of_signal() with multi-id version, that samples the requested data from pipe,
//  stores that in the iterator and returns to caller.
//  - ✓ build_table() needs to request all rows in one go and iterate over them.
//  - ✓ (opt) Add a LRU cache as pipeline stage on values using cycle and id as tag. Invalidate on
//  reload. Limit in size.
//  - ✓ VCD loader parses whole file, but only allocates data for requested range.
pub struct Wave 
{
    formatters: Vec<WaveFormat>,
    names: Vec<String>,
    pipe: PipelineCId,
    config: PipelineConfig,
    num_signals: usize,
    cache: Cache,
}

impl Wave {
    pub fn load(source: SrcBox/*, config: &Config*/) -> Result<Self> {
        let pipe = PipelineCId::new(source)?;
        let config = PipelineConfig::default();
        Self::load_from_pipe(pipe, config)
    }

    fn load_from_pipe(mut pipe: PipelineCId, config: PipelineConfig) -> Result<Self> {
        pipe.query_init()?;
        let signals = pipe.query_signals()?;
        let num_signals = signals.len();
        let mut ids = Vec::with_capacity(signals.len());
        let mut names = Vec::with_capacity(signals.len());
        let mut formatters = Vec::with_capacity(signals.len());
        for signal in signals {
            let Signal { id, name, format } = signal;
            ids.push(id);
            names.push(name);
            formatters.push(format);
        }

        let num_cycles = pipe.query_cycle_count();
        // TODO use config object
        let cache = Cache::new(128, 128, 1024, num_signals, num_cycles
            //config.wave_cache_capacity(),
            //config.wave_cache_signals_per_tile(),
            //config.wave_cache_cycles_per_tile()
        );

        Ok(Self {
            formatters,
            names,
            pipe,
            config,
            num_signals,
            cache
        })
    }

    pub fn num_cycles(&self) -> usize {
        self.pipe.query_cycle_count()
    }

    pub fn num_signals(&self) -> usize {
        self.num_signals
    }

    /// Return an interval [left, right) of cycles from the wave
    //pub fn slice(&self, ids: std::ops::Range<usize>, cycles: std::ops::Range<usize>) -> Result<WaveSlice> {
        //let a = self.pipe.query_time(cycles.start);
        //let b = self.pipe.query_time(cycles.end);
        //let id_vec = ids.clone().collect();
        //let data = self.pipe.sample(&id_vec, &SimTimeRange(a, b))?;
        
        //Ok(WaveSlice {
            //data,
            //names: &self.names,
            //formatters: &self.formatters,
            //cycles,
            //ids,
        //})
    //}

    /// Return a slice from the cache
    ///
    /// LRU cache over blocks of data, e.g. 128x1024. Use sample to get those individually.
    /// Pick from cache and copy to WaveSlice.
    pub fn cached_slice(&mut self, ids: std::ops::Range<usize>, cycles: std::ops::Range<usize>) -> Result<WaveSlice> {
        let mut data = Array2::default((cycles.len(), ids.len()));

        for (i,id) in ids.clone().enumerate() {
            data.slice_mut(s![.., i]).assign(&self.cache.get(&mut self.pipe, id, cycles.clone()));
        }

        Ok(WaveSlice {
            data,
            names: &self.names,
            formatters: &self.formatters,
            cycles,
            ids
        })
    }

    pub fn formatter(&self, signal_index: usize) -> WaveFormat {
        self.formatters[signal_index]
    }

    pub fn set_formatter(&mut self, signal_index: usize, format: WaveFormat) {
        self.formatters[signal_index] = format;
    }

    pub fn value(&mut self, signal_index: usize, cycle: usize) -> Option<Integer> {
        let wave_slice = self.cached_slice(signal_index..signal_index+1, cycle..cycle+1).ok()?;
        wave_slice.value(signal_index, cycle)
            .map(|x| x.clone())
    }

    pub fn formatted_value(&mut self, signal_index: usize, cycle: usize) -> Option<String> {
        self.value(signal_index, cycle)
            .map(|val| {
                let format = self.formatters[signal_index];
                format_value(&val, format)
            })
    }

    pub fn name<'a>(&'a self, signal_index: usize) -> Option<&'a str> {
        self.names
            .get(signal_index)
            .map(|s| s.as_str())
    }

    pub fn get_names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn push_filter(self, filter: FilterBox) -> Result<Self> {
        Self::load_from_pipe(self.pipe.push(filter), self.config)
    }

    pub fn pop_filter(self) -> Result<(Self, Option<FilterBox>)> {
        let (pipe, filter) = self.pipe.pop();
        let new_self = Self::load_from_pipe(pipe, self.config)?;

        Ok((new_self, filter))
    }

    pub fn get_config_mut(&mut self) -> &mut PipelineConfig {
        &mut self.config
    }

    pub fn reconfigure(&mut self) -> Result<()> {
        self.pipe.configure_pipeline(&self.config)
    }

    pub fn reload(self) -> Result<Self> {
        Self::load_from_pipe(self.pipe, self.config)
    }

    /// Find the next transition for a single signal
    ///
    /// * `signal_index` - Row of the signal
    /// * `start_cycle` - First cycle within row to begin search
    ///
    /// Find the next cycle of the current signal's trace that is not equal to the value at
    /// `start_cycle`.
    pub fn cached_next_transition(&mut self, signal_index: usize, mut start_cycle: usize) -> Option<usize> {
        let mut horizon = std::cmp::min(
            start_cycle + SEARCH_HORIZON,
            self.num_cycles()
        );

        while horizon <= self.num_cycles() {
            let wave_slice = self.cached_slice(signal_index..signal_index+1, start_cycle..horizon).ok()?;
            if let Some(found) = wave_slice.next_transition(signal_index, start_cycle) {
                return Some(found);
            }

            start_cycle = horizon;
            horizon += SEARCH_HORIZON;
        }

        None
    }

    /// Find the first previous transition for a single signal
    ///
    /// * `signal_index` - Row of the signal
    /// * `start_cycle` - First cycle within row to begin search
    ///
    /// Find the first preceding cycle of the current signal's trace that is not equal to the value
    /// at `start_cycle`.
    pub fn cached_prev_transition(&mut self, signal_index: usize, mut start_cycle: usize) -> Option<usize> {
        let mut horizon = start_cycle.saturating_sub(SEARCH_HORIZON);

        loop {
            let wave_slice = self.cached_slice(signal_index..signal_index+1, horizon..start_cycle + 1).ok()?;
            if let Some(found) = wave_slice.prev_transition(signal_index, start_cycle) {
                return Some(found);
            }

            if horizon == 0 {
                return None;
            }

            start_cycle = horizon;
            horizon = horizon.saturating_sub(SEARCH_HORIZON);
        }

        //let wave_slice = self.cached_slice(signal_index..signal_index+1, 0..start_cycle+1).ok()?;
        //wave_slice.prev_transition(signal_index, start_cycle)
    }
}

/// Owns data of a collection of signals in an interval of cycles
pub struct WaveSlice<'a> {
    data: Array2<Integer>,
    names: &'a Vec<String>,
    formatters: &'a Vec<WaveFormat>,
    cycles: std::ops::Range<usize>,
    ids: std::ops::Range<usize>,
}

impl<'a> WaveSlice<'a> {
    /// Return iterator over data of a single signal
    pub fn signal_iter(&self, i: usize) -> Result<SliceIter> {
        if !self.ids.contains(&i) {
            Err(Error::IdOutOfRange(i, self.ids.clone()))
        } else {
            Ok(SliceIter {
                data: &self.data,
                ptr: 0,
                end: self.cycles.len(),
                signal_index: i - self.ids.start,
            })
        }
    }

    pub fn formatter(&self, signal_index: usize) -> WaveFormat {
        self.formatters[signal_index]
    }

    pub fn name(&self, signal_index: usize) -> Option<&'a str> {
        self.names
            .get(signal_index)
            .map(|s| s.as_str())
    }

    pub fn value(&self, signal_index: usize, cycle: usize) -> Option<&Integer> {
        self.data.get([cycle - self.cycles.start, signal_index - self.ids.start])
    }

    pub fn formatted_value(&self, signal_index: usize, cycle: usize) -> Option<String> {
        self.value(signal_index, cycle)
            .map(|val| {
                let format = self.formatters[signal_index];
                format_value(&val, format)
            })
    }

    /// Find the next transition for a single signal
    ///
    /// * `signal_index` - Row of the signal
    /// * `start_cycle` - First cycle within row to begin search
    ///
    /// Find the next cycle of the current signal's trace that is not equal to the value at
    /// `start_cycle`.
    pub fn next_transition(&self, signal_index: usize, start_cycle: usize) -> Option<usize> {
        if self.ids.contains(&signal_index) && self.cycles.contains(&start_cycle) {
            let slice_index = signal_index - self.ids.start;
            let slice_cycle = start_cycle - self.cycles.start;

            let col = self.data.column(slice_index);
            let cur_val = &col[slice_cycle];
            col.slice(s![slice_cycle..])
                .iter()
                .position(|x| *x != *cur_val)
                .map(|x| x + start_cycle)
        } else {
            None
        }
    }


    /// Find the first previous transition for a single signal
    ///
    /// * `signal_index` - Row of the signal
    /// * `start_cycle` - First cycle within row to begin search
    ///
    /// Find the first preceding cycle of the current signal's trace that is not equal to the value
    /// at `start_cycle`.
    pub fn prev_transition(&self, signal_index: usize, start_cycle: usize) -> Option<usize> {
        if self.ids.contains(&signal_index) && self.cycles.contains(&start_cycle) {
            let slice_index = signal_index - self.ids.start;
            let slice_cycle = start_cycle - self.cycles.start;

            let col = self.data.column(slice_index);
            let cur_val = &col[slice_cycle];
            col.slice(s![0..slice_cycle+1; -1])
                .iter()
                .position(|x| *x != *cur_val)
                .map(|offset| start_cycle - offset)
        } else {
            None
        }
    }
}

/// Iterator over data belonging to a single signal
pub struct SliceIter<'a> {
    data: &'a Array2<Integer>,
    ptr: usize,
    end: usize,
    signal_index: usize,
}


impl<'a> Iterator for SliceIter<'a> {
    type Item = &'a Integer;

    fn next(&mut self) -> Option<Self::Item> {
        let rv;

        if self.ptr <= self.end {
            rv = self.data.get([self.ptr, self.signal_index]);
            self.ptr += 1;
        } else {
            rv = None
        }

        rv
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::load::vcd::VcdLoader;

    fn make_test_wave() -> Result<Wave> {
        let loader = Box::new(VcdLoader::new("examples/verilator.vcd", Some(SimTime::from_ps(1)))?);
        let wave = Wave::load(loader)?;

        Ok(wave)
    }

    #[test]
    fn test_example_wave_data() {
        let mut wave = make_test_wave()
            .expect("Failed to load test wave data");

        assert_eq!(16, wave.num_signals());
        assert_eq!(211, wave.num_cycles());

        assert_eq!(Some(Integer::from(0)), wave.value(7, 0));
        assert_eq!(Some(Integer::from(1)), wave.value(7, 1));
        assert_eq!(Some(Integer::from(1)), wave.value(7, 40));
        assert_eq!(Some(Integer::from(0)), wave.value(7, 41));

        let wave_slice = wave.cached_slice(0..wave.num_signals(), 0..wave.num_cycles()).unwrap();
        let col = wave_slice.data.column(7);

        assert_eq!(Integer::from(0), col[0]);
        assert_eq!(Integer::from(1), col[1]);
        assert_eq!(Integer::from(1), col[40]);
        assert_eq!(Integer::from(0), col[41]);
    }

    #[test]
    fn test_transitions() {
        let mut wave = make_test_wave()
            .expect("Failed to load test wave data");

        assert_eq!(Some(1), wave.cached_next_transition(7, 0));
        assert_eq!(Some(41), wave.cached_next_transition(7, 1));

        assert_eq!(Some(0), wave.cached_prev_transition(7, 40));
    }

    #[test]
    fn test_wave_slice() {
        let mut wave = make_test_wave()
            .expect("Failed to load test wave data");

        let wave_slice = wave.cached_slice(7..8, 0..50).unwrap();
        let data: Vec<_> = wave_slice.signal_iter(7)
            .unwrap()
            .collect();

        assert_eq!(Integer::from(0), *data[0]);
        assert_eq!(Integer::from(1), *data[1]);
        assert_eq!(Integer::from(1), *data[40]);
        assert_eq!(Integer::from(0), *data[41]);


        let wave_slice = wave.cached_slice(0..8, 39..53).unwrap();
        let data: Vec<_> = wave_slice.signal_iter(7)
            .unwrap()
            .collect();

        assert_eq!(Integer::from(1), *data[1]);
        assert_eq!(Integer::from(0), *data[2]);
    }
}
