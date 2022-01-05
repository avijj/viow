use super::*;
use crate::formatting::WaveFormat;
use crate::data::*;

use rug::Assign;
use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use ::vcd::{ self, Parser, ScopeItem, Header, Value };
use ndarray::prelude::*;

struct SignalInfo {
    index: usize,
    var_type: vcd::VarType,
}

type SignalMap = HashMap<vcd::IdCode, SignalInfo>;

pub struct VcdLoader {
    signals: Vec<SignalDeclaration>,
    data: Vec<Vec<Integer>>,
    num_cycles: usize,
    cycle_time: SimTime,
}

impl VcdLoader {
    pub fn new(filename: impl AsRef<Path>, cycle_time: SimTime) -> Result<Self> {
        let file = File::open(filename.as_ref())?;
        let mut parser = Parser::new(file);

        let header = parser.parse_header()?;
        let (signals, ids) = Self::load_all_scopes(&header);
        let data = Self::load_all_waveforms(&mut parser, &ids, signals.len(), cycle_time);

        let num_cycles= data.len();

        Ok(Self {
            signals,
            data,
            num_cycles,
            cycle_time
        })
    }

    fn load_all_scopes(header: &Header) -> (Vec<SignalDeclaration>, SignalMap) {
        let mut rv = vec![];
        let mut stack = vec![("".to_string(), &header.items)];
        let mut sigmap = SignalMap::new();

        loop {
            if let Some((prefix, scope)) = stack.pop() {
                for item in scope.iter() {
                    match item {
                        ScopeItem::Var(var) => {
                            let name = format!("{}{}", prefix, var.reference);
                            let format = if var.size == 1 {
                                WaveFormat::Bit
                            } else {
                                WaveFormat::Vector
                            };

                            rv.push(SignalDeclaration { name, format });

                            let info = SignalInfo {
                                index: rv.len()-1,
                                var_type: var.var_type,
                            };

                            sigmap.insert(var.code, info);
                        }

                        ScopeItem::Scope(sub_scope) => {
                            let new_prefix = format!("{}{}.", prefix, sub_scope.identifier);
                            stack.push((new_prefix, &sub_scope.children));
                        }

                        ScopeItem::Comment(comment) => {
                            rv.push(SignalDeclaration {
                                name: format!("-- {}", comment),
                                format: WaveFormat::Comment,
                            });
                        }
                    }
                }
            } else {
                break;
            }
        }

        (rv, sigmap)
    }

    fn map_values_to_int(target: &mut Integer, x: &Value) {
        match *x {
            Value::V1 => target.assign(1),
            _ => target.assign(0)
        }
    }

    fn map_vec_to_int(target: &mut Integer, x: &Vec<Value>) {
        target.assign(0);
        for (i,bit) in x.iter().enumerate() {
            let val = match *bit {
                Value::V1 => true,
                _ => false
            };
            
            target.set_bit((x.len() - 1 - i) as u32, val);
        }
    }

    fn timescale_to_simtime(ts: u32, unit: vcd::TimescaleUnit) -> SimTime {
        use vcd::TimescaleUnit::*;
        let u = match unit {
            S => SimTimeUnit::S,
            MS => SimTimeUnit::Ms,
            US => SimTimeUnit::Us,
            NS => SimTimeUnit::Ns,
            PS => SimTimeUnit::Ps,
            FS => SimTimeUnit::Fs,
        };

        SimTime::new(ts as u64, u)
    }

    fn load_all_waveforms<T: std::io::Read>(parser: &mut Parser<T>,
        ids: &SignalMap,
        num_signals: usize,
        cycle_time: SimTime 
    ) -> Vec<Vec<Integer>> {
        let mut rv: Vec<Vec<Integer>> = vec![];
        let mut vals = vec![Integer::default(); num_signals];
        let mut cur_t = 0;
        let mut cycle_time_ts: u64 = cycle_time / SimTime::from_ps(1);

        for command in parser {
            if command.is_err() {
                continue;
            }

            let command = command.unwrap();

            use vcd::Command::*;
            match command {
                Timescale(ts, unit) => {
                    let timescale = Self::timescale_to_simtime(ts, unit);
                    cycle_time_ts = cycle_time / timescale;
                }

                Timestamp(t) => {
                    if (t - cur_t) >= cycle_time_ts {
                        let ints: Vec<_> = vals.iter()
                            .map(|x| x.clone())
                            //.map(Self::map_values_to_int)
                            .collect();
                        rv.push(ints);
                        cur_t += cycle_time_ts;
                    }
                }

                ChangeScalar(i, v) => {
                    if let Some(info) = ids.get(&i) {
                        Self::map_values_to_int(&mut vals[info.index], &v);
                    }
                }

                ChangeVector(i, v) => {
                    if let Some(info) = ids.get(&i) {
                        Self::map_vec_to_int(&mut vals[info.index], &v);
                    }
                }

                _ => ()
            }
        }

        rv
    }
}


impl LoadDeclarations for VcdLoader {
    fn load_declarations(&self) -> Result<Vec<SignalDeclaration>> {
        Ok(self.signals.clone())
    }
}


impl LoadLength for VcdLoader {
    fn load_length(&self) -> Result<usize> {
        Ok(self.num_cycles)
    }
}


impl LoadWaveform for VcdLoader {
    fn load_waveform(&self, name: impl AsRef<str>, cycles: Range<usize>) -> Result<Vec<Integer>> {
        let mut rv = Vec::with_capacity(cycles.len());

        let pos = self.signals.iter()
            .position(|x| x.name == name.as_ref());

        if let Some(pos) = pos {
            if cycles.end > self.num_cycles {
                Err(Error::InvalidRange(cycles, 0..self.num_cycles))
            } else {
                for cycle in cycles {
                    rv.push(self.data[cycle][pos].clone());
                }

                Ok(rv)
            }
        } else {
            Err(Error::NotFound(name.as_ref().to_string()))
        }
    }
}

// 
// new style traits
//

impl QuerySource for VcdLoader {
    type Id = String;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let rv: Self::IntoSignalIter = self.signals.iter()
            .map(|decl| Signal {
                id: decl.name.clone(),
                name: decl.name.clone(),
                format: decl.format.clone()
            })
            .collect();

        Ok(rv)
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        let start = SimTime::zero();
        let stop = self.cycle_time * (self.num_cycles as u64);

        Ok(SimTimeRange(start, stop))
    }
}

impl LookupId for VcdLoader {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: &Self::FromId) -> Result<Self::ToId> {
        let pos = self.signals.iter()
            .position(|x| x.name == *id);

        match pos {
            Some(p) => Ok(p),
            None => Err(Error::NotFound(id.clone()))
        }
    }

    fn rev_lookup_id(&self, id: &Self::ToId) -> Result<Self::FromId> {
        if *id < self.signals.len() {
            Ok(self.signals[*id].name.clone())
        } else {
            Err(Error::IdOutOfRange(*id, self.signals.len()-1))
        }
    }
}

impl Sample for VcdLoader {
    type Id = String;
    type Value = Integer;

    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
        let start_cycle = (times.0 / self.cycle_time) as usize;
        let stop_cycle = (times.1 / self.cycle_time) as usize;

        let mut data = Array2::default((stop_cycle - start_cycle, ids.len()));
        for (i, id) in ids.iter().enumerate() {
            let wv = self.load_waveform(id, start_cycle..stop_cycle)?;
            data.slice_mut(s![..,i]).assign(&Array1::from_vec(wv));
        }

        Ok(data)
    }
}

impl Source<String, usize, Integer> for VcdLoader {}

