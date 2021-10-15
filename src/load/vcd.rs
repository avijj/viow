use super::*;
use crate::formatting::WaveFormat;

use rug::{Assign,integer::Order};
use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use ::vcd::{ self, Parser, ScopeItem, Header, Value };

struct SignalInfo {
    index: usize,
    var_type: vcd::VarType,
}

type SignalMap = HashMap<vcd::IdCode, SignalInfo>;

pub struct VcdLoader {
    signals: Vec<SignalDeclaration>,
    data: Vec<Vec<Integer>>,
    num_cycles: usize,
}

impl VcdLoader {
    pub fn new(filename: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(filename.as_ref())?;
        let mut parser = Parser::new(file);

        let header = parser.parse_header()?;
        let (signals, ids) = Self::load_all_scopes(&header);
        let data = Self::load_all_waveforms(&mut parser, &ids, signals.len(), 100000);

        let num_cycles= data.len();

        Ok(Self {
            signals,
            data,
            num_cycles
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
                            let format = WaveFormat::Vector;

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

    fn load_all_waveforms<T: std::io::Read>(parser: &mut Parser<T>,
        ids: &SignalMap,
        num_signals: usize,
        cycle_time: u64
    ) -> Vec<Vec<Integer>> {
        let mut rv: Vec<Vec<Integer>> = vec![];
        let mut vals = vec![Integer::default(); num_signals];
        let mut cur_t = 0;

        let ints: Vec<Integer> = vals.iter()
            .map(|x| x.clone())
            //.map(Self::map_values_to_int)
            .collect();
        rv.push(ints);

        for command in parser {
            if command.is_err() {
                continue;
            }

            let command = command.unwrap();

            use vcd::Command::*;
            match command {
                Timestamp(t) => {
                    if (t - cur_t) >= cycle_time {
                        let ints: Vec<_> = vals.iter()
                            .map(|x| x.clone())
                            //.map(Self::map_values_to_int)
                            .collect();
                        rv.push(ints);
                        cur_t += cycle_time;
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

