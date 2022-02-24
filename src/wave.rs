use crate::error::*;
use crate::formatting::{WaveFormat,format_value};
use crate::data::*;
use crate::pipeline::*;

use ndarray::prelude::*;
use ndarray;
use rug::Integer;

pub struct Wave 
{
    data: Array2<Integer>,
    formatters: Vec<WaveFormat>,
    names: Vec<String>,
    pipe: Pipeline,
    config: PipelineConfig,
}

impl Wave {
    pub fn load(source: SrcBox) -> Result<Self> {
        let pipe = Pipeline::new(source);
        let config = PipelineConfig::default();
        Self::load_from_pipe(pipe, config)
    }

    fn load_from_pipe(pipe: Stage<String, usize, Integer>, config: PipelineConfig) -> Result<Self> {
        let signals = pipe.query_signals()?;
        let mut ids = Vec::with_capacity(signals.len());
        let mut names = Vec::with_capacity(signals.len());
        let mut formatters = Vec::with_capacity(signals.len());
        for signal in signals {
            let Signal { id, name, format } = signal;
            ids.push(id);
            names.push(name);
            formatters.push(format);
        }
        let times = pipe.query_time()?;
        let data = pipe.sample(&ids, &times)?;

        Ok(Self {
            data,
            formatters,
            names,
            pipe,
            config,
        })
    }

    pub fn num_cycles(&self) -> usize {
        self.data.dim().0
    }

    pub fn num_signals(&self) -> usize {
        self.data.dim().1
    }

    pub fn slice_of_signal(&self,
        i: usize,
        left: usize,
        right: usize
    ) -> SliceIter {
        SliceIter {
            data: &self.data,
            ptr: left,
            end: right,
            signal_index: i,
        }
    }

    pub fn formatter(&self, signal_index: usize) -> WaveFormat {
        self.formatters[signal_index]
    }

    pub fn set_formatter(&mut self, signal_index: usize, format: WaveFormat) {
        self.formatters[signal_index] = format;
    }

    pub fn value<'a>(&'a self, signal_index: usize, cycle: usize) -> Option<&'a Integer> {
        self.data.get([cycle, signal_index])
    }

    pub fn formatted_value(&self, signal_index: usize, cycle: usize) -> Option<String> {
        self.value(signal_index, cycle)
            .map(|val| {
                let format = self.formatters[signal_index];
                format_value(val, format)
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
    pub fn next_transition(&self, signal_index: usize, start_cycle: usize) -> Option<usize> {
        let col = self.data.column(signal_index);
        let cur_val = &col[start_cycle];
        col.slice(s![start_cycle..])
            .iter()
            .position(|x| *x != *cur_val)
            .map(|x| x + start_cycle)
    }

    /// Find the first previous transition for a single signal
    ///
    /// * `signal_index` - Row of the signal
    /// * `start_cycle` - First cycle within row to begin search
    ///
    /// Find the first preceding cycle of the current signal's trace that is not equal to the value
    /// at `start_cycle`.
    pub fn prev_transition(&self, signal_index: usize, start_cycle: usize) -> Option<usize> {
        let col = self.data.column(signal_index);
        let cur_val = &col[start_cycle];
        col.slice(s![0..start_cycle+1; -1])
            .iter()
            .position(|x| *x != *cur_val)
            .map(|offset| start_cycle - offset)
    }
}



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
        let wave = make_test_wave()
            .expect("Failed to load test wave data");

        assert_eq!(Some(&Integer::from(0)), wave.value(7, 0));
        assert_eq!(Some(&Integer::from(1)), wave.value(7, 1));
        assert_eq!(Some(&Integer::from(1)), wave.value(7, 40));
        assert_eq!(Some(&Integer::from(0)), wave.value(7, 41));

        let col = wave.data.column(7);

        assert_eq!(Integer::from(0), col[0]);
        assert_eq!(Integer::from(1), col[1]);
        assert_eq!(Integer::from(1), col[40]);
        assert_eq!(Integer::from(0), col[41]);
    }

    #[test]
    fn test_transitions() {
        let wave = make_test_wave()
            .expect("Failed to load test wave data");

        assert_eq!(Some(1), wave.next_transition(7, 0));
        assert_eq!(Some(41), wave.next_transition(7, 1));

        assert_eq!(Some(0), wave.prev_transition(7, 40));
    }
}
