use crate::error::*;
use crate::formatting::{WaveFormat};
use crate::load::*;
use crate::data::*;
use crate::pipeline::*;

use ndarray::prelude::*;
use ndarray;
use rug::Integer;

pub struct Wave<S> 
{
    data: Array2<Integer>,
    formatters: Vec<WaveFormat>,
    names: Vec<String>,
    pipe: Pipeline<S>,
}

impl<S> Wave<S>
    where
        S: Source<String, rug::Integer> + LookupId<FromId = String, ToId = usize>
{
    /*pub fn _new() -> Self {
        //let mut data = vec![vec![Integer::from(0); 200]];
        //let mut data = Array2::<Integer>::zeros((1000, 200));
        let mut data = Array2::<Integer>::from_elem((1000, 200), Integer::from(0));
        let mut formatters = vec![WaveFormat::Bit; 200];
        data.slice_mut(s![..,1]).fill(Integer::from(1));
        data.slice_mut(s![..;2,2]).fill(Integer::from(1));

        let counter: Vec<Integer> = (0..data.dim().0).into_iter()
            .map(|x: usize| Integer::from((x >> 2) % 16))
            .collect();
        data.slice_mut(s![..,4]).assign(&Array1::from_vec(counter));
        formatters[4] = WaveFormat::Vector;

        let counter: Vec<Integer> = (0x4000..data.dim().0 + 0x4000).into_iter()
            .map(|x: usize| Integer::from((x >> 2) % 0x10000))
            .collect();
        data.slice_mut(s![..,5]).assign(&Array1::from_vec(counter));
        formatters[5] = WaveFormat::Vector;

        let names: Vec<_> = (0..data.dim().1)
            .map(|i| format!("row_{}", i))
            .collect();

        Self {
            data,
            formatters,
            names
        }
    }*/

    pub fn load_new(source: S, source_adapter: SourceAdapter, exit_adapter: ExitAdapter) -> Result<Self> {
        let pipe = Pipeline::new(source, source_adapter, exit_adapter);
        let signals = pipe.query_signals()?;
        let mut names = Vec::with_capacity(signals.len());
        let mut formatters = Vec::with_capacity(signals.len());

        for signal in signals {
            let (name, format) = signal;
            names.push(name);
            formatters.push(format);
        }

        let times = pipe.query_time()?;
        let data = pipe.sample(&names, &times)?;

        Ok(Self {
            data,
            formatters,
            names,
            pipe,
        })
    }

    pub fn _load<T>(loader: &T) -> Result<Self>
        where
            T: LoadDeclarations + LoadLength + LoadWaveform
    {
        let decls = loader.load_declarations()?;
        let num_cycles = loader.load_length()?;
        let mut data = Array2::default((num_cycles, decls.len()));
        let mut formatters = Vec::with_capacity(decls.len());
        let mut names = Vec::with_capacity(decls.len());
       
        for (i, decl) in decls.into_iter().enumerate() {
            let wv = loader.load_waveform(&decl.name, 0..num_cycles)?;
            data.slice_mut(s![..,i]).assign(&Array1::from_vec(wv));

            formatters.push(decl.format);
            names.push(decl.name);
        }

        unimplemented!();
        //Ok(Self {
            //data,
            //formatters,
            //names,
        //})
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

