use crate::error::*;
use crate::formatting::WaveFormat;
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

    pub fn value<'a>(&'a self, signal_index: usize, cycle: usize) -> Option<&'a Integer> {
        self.data.get([cycle, signal_index])
    }

    pub fn name<'a>(&'a self, signal_index: usize) -> Option<&'a str> {
        self.names
            .get(signal_index)
            .map(|s| s.as_str())
    }

    pub fn push_filter(self, filter: FilterBox) -> Result<Self> {
        Self::load_from_pipe(self.pipe.push(filter), self.config)
    }

    pub fn pop_filter(self) -> Result<(Self, Option<FilterBox>)> {
        let (pipe, filter) = self.pipe.pop();
        let new_self = Self::load_from_pipe(pipe, self.config)?;

        Ok((new_self, filter))
    }

    pub fn get_config(&self) -> &PipelineConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut PipelineConfig {
        &mut self.config
    }

    pub fn reconfigure(&mut self) -> Result<()> {
        self.pipe.configure_pipeline(&self.config)
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

