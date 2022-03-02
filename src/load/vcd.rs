use crate::data::*;
use crate::formatting::WaveFormat;
use crate::error::*;

use rug::Integer;
use std::ops::Range;

use ::vcd::{self, Header, Parser, ScopeItem, Value};
use ndarray::prelude::*;
use rug::Assign;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path,PathBuf};

struct SignalInfo {
    index: usize,
    size: u32,
}

struct Subset {
    data: Array2<vcd::Value>,
    bitmap: SignalBitMap,
}

#[derive(Clone)]
struct SignalDeclaration {
    pub name: String,
    pub format: WaveFormat,
}

type SignalMap = HashMap<vcd::IdCode, SignalInfo>;
type SignalBitMap = HashMap<vcd::IdCode, std::ops::Range<usize>>;
type NameMap = HashMap<String, vcd::IdCode>;

pub struct VcdLoader {
    filename: PathBuf,
    signals: Vec<SignalDeclaration>,
    num_cycles: usize,
    cycle_time: SimTime,
}

impl VcdLoader {
    pub fn new(filename: impl AsRef<Path>, cycle_time: Option<SimTime>) -> Result<Self> {
        let file = File::open(filename.as_ref())?;
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;
        let timescale = header
            .timescale
            .map(|(n, ts)| Self::timescale_to_simtime(n, ts))
            .unwrap_or(SimTime::from_ps(1));
        let (signals, _ids, _) = Self::load_all_scopes(&header);
        let cycle_time = cycle_time.unwrap_or(timescale);

        let num_cycles = Self::load_count_cycles(&mut parser, cycle_time, timescale);

        Ok(Self {
            filename: filename.as_ref().into(),
            signals,
            num_cycles,
            cycle_time,
        })
    }

    fn load_all_scopes(header: &Header) -> (Vec<SignalDeclaration>, SignalMap, NameMap) {
        let mut rv = vec![];
        let mut stack = vec![("".to_string(), &header.items)];
        let mut sigmap = SignalMap::new();
        let mut namemap = NameMap::new();

        loop {
            if let Some((prefix, scope)) = stack.pop() {
                for item in scope.iter() {
                    match item {
                        ScopeItem::Var(var) => {
                            let name = format!("{}{}", prefix, var.reference);
                            let format = if var.size == 1 {
                                WaveFormat::Bit
                            } else {
                                WaveFormat::Vector(var.size)
                            };

                            namemap.insert(name.clone(), var.code);
                            rv.push(SignalDeclaration { name, format });

                            let info = SignalInfo {
                                index: rv.len() - 1,
                                size: var.size,
                            };

                            sigmap.insert(var.code, info);
                        }

                        ScopeItem::Scope(sub_scope) => {
                            let new_prefix = format!("{}{}.", prefix, sub_scope.identifier);
                            stack.push((new_prefix, &sub_scope.children));
                        }

                        ScopeItem::Comment(comment) => {
                            let name = format!("-- {}: {}", prefix.strip_suffix(".").unwrap_or(""), comment);
                            rv.push(SignalDeclaration {
                                name,
                                format: WaveFormat::Comment,
                            });
                        }
                    }
                }
            } else {
                break;
            }
        }

        (rv, sigmap, namemap)
    }

    fn map_values_to_int(target: &mut Integer, x: &Value) {
        match *x {
            Value::V1 => target.assign(1),
            _ => target.assign(0),
        }
    }

    fn map_vec_to_int(target: &mut Integer, x: &Vec<Value>) {
        target.assign(0);
        for (i, bit) in x.iter().enumerate() {
            let val = match *bit {
                Value::V1 => true,
                _ => false,
            };

            target.set_bit((x.len() - 1 - i) as u32, val);
        }
    }

    fn map_array_to_int<'a>(target: &mut Integer, x: impl AsArray<'a, vcd::Value>) {
        target.assign(0);
        let ar = x.into();
        for (i, bit) in ar.iter().enumerate() {
            let val = match *bit {
                Value::V1 => true,
                _ => false,
            };

            target.set_bit((ar.len() - 1 - i) as u32, val);
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

    fn load_count_cycles<T: std::io::Read>(
        parser: &mut Parser<T>,
        cycle_time: SimTime,
        timescale: SimTime,
    ) -> usize {
        let mut cur_t = 0;
        let mut cur_cycle = 0;
        let mut cycle_time_ts: u64 = cycle_time / timescale;

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
                    while (t - cur_t) >= cycle_time_ts {
                        cur_t += cycle_time_ts;
                        cur_cycle += 1;
                    }
                }

                _ => (),
            }
        }

        cur_cycle
    }

    fn load_all_waveforms<T: std::io::Read>(
        parser: &mut Parser<T>,
        ids: &SignalMap,
        num_signals: usize,
        cycle_time: SimTime,
        timescale: SimTime,
    ) -> Vec<Vec<Integer>> {
        let mut rv: Vec<Vec<Integer>> = vec![];
        let mut vals = vec![Integer::default(); num_signals];
        let mut cur_t = 0;
        let mut cycle_time_ts: u64 = cycle_time / timescale;

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
                    while (t - cur_t) >= cycle_time_ts {
                        let ints: Vec<_> = vals.iter().map(|x| x.clone()).collect();
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

                _ => (),
            }
        }

        rv
    }

    fn assign_bit_positions(signals: &SignalMap, record_ids: &[vcd::IdCode]) -> Result<(SignalBitMap, usize)> {
        let mut ptr = 0;
        let mut rv = SignalBitMap::new();

        for id in record_ids.iter() {
            let info = signals.get(id)
                .ok_or(Error::Internal(format!("signal {} was not found in signal map", id)))?;

            rv.insert(*id, ptr..ptr + (info.size as usize));
            ptr += info.size as usize;
        }

        Ok((rv, ptr))
    }

    fn load_subset<T: std::io::Read>(
        parser: &mut Parser<T>,
        ids: &SignalMap,
        cycle_time: SimTime,
        timescale: SimTime,
        record_ids: &[vcd::IdCode],
        record_cycles: std::ops::Range<u64>,
    ) -> Result<Subset> {
        // construct <cycles> x <signals> array for result data
        let (bitmap, width) = Self::assign_bit_positions(ids, record_ids)?;
        let height = (record_cycles.end - record_cycles.start) as usize;
        let mut data = Array2::from_elem((height, width), vcd::Value::X);
        let mut cur = Array1::from_elem(width, Value::X);
        let mut cur_cycle: u64 = 0;
        let mut cur_t = 0;
        let mut cycle_time_ts: u64 = cycle_time / timescale;

        'command_loop: for command in parser {
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
                    while (t - cur_t) >= cycle_time_ts {
                        if record_cycles.contains(&cur_cycle) {
                            let rel_cycle = (cur_cycle - record_cycles.start) as usize;
                            data.slice_mut(s![rel_cycle, ..]).assign(&cur);
                        } else if cur_cycle >= record_cycles.end {
                            // early exit when all requested data is recorded
                            break 'command_loop;
                        }

                        cur_t += cycle_time_ts;
                        cur_cycle += 1;
                    }
                }

                ChangeScalar(i, v) => {
                    if let Some(bitrange) = bitmap.get(&i) {
                        cur[[bitrange.start]] = v;
                    }
                }

                ChangeVector(i, v) => {
                    if let Some(bitrange) = bitmap.get(&i) {
                        cur.slice_mut(s![bitrange.clone()])
                            .assign(&Array1::from_vec(v));
                    }
                }

                _ => (),
            }
        }

        let rv = Subset {
            data,
            bitmap
        };

        Ok(rv)
    }
}


//
// new style traits
//

impl QuerySource for VcdLoader {
    type Id = String;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let rv: Self::IntoSignalIter = self
            .signals
            .iter()
            .map(|decl| Signal {
                id: decl.name.clone(),
                name: decl.name.clone(),
                format: decl.format.clone(),
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

impl LookupId for VcdLoader {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: &Self::FromId) -> Result<Self::ToId> {
        let pos = self.signals.iter().position(|x| x.name == *id);

        match pos {
            Some(p) => Ok(p),
            None => Err(Error::NotFound(id.clone())),
        }
    }

    fn rev_lookup_id(&self, id: &Self::ToId) -> Result<Self::FromId> {
        if *id < self.signals.len() {
            Ok(self.signals[*id].name.clone())
        } else {
            Err(Error::IdOutOfRange(*id, 0..self.signals.len()))
        }
    }
}

impl Sample for VcdLoader {
    type Id = String;
    type Value = Integer;

    fn sample(
        &self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        let start_cycle = times.0 / self.cycle_time;
        let stop_cycle = times.1 / self.cycle_time;

        // load data from file
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;
        let timescale = header
            .timescale
            .map(|(n, ts)| Self::timescale_to_simtime(n, ts))
            .unwrap_or(SimTime::from_ps(1));
        let (_signals, info, namemap) = Self::load_all_scopes(&header);

        // translate to VCD Ids
        let record_ids: Vec<vcd::IdCode> = ids.iter()
            .filter_map(|id| {
                namemap.get(id)
            })
            .map(|x| x.clone())
            .collect();

        // load subset
        let subset = Self::load_subset(&mut parser, &info, self.cycle_time, timescale, &record_ids,
            start_cycle..stop_cycle)?;

        // convert to Integer
        let num_cycles = (stop_cycle - start_cycle) as usize;
        let num_signals = ids.len();
        let mut data = Array2::default((num_cycles, num_signals));

        for (row_i, mut row) in data.outer_iter_mut().enumerate() {
            for (col_i, name) in ids.iter().enumerate() {
                if let Some(idcode) = namemap.get(name) {
                    let bitrange = subset.bitmap.get(idcode)
                        .ok_or(Error::Internal(format!("Could not find bit position of VCD IdCode '{}'", idcode)))?;
                    let bits = subset.data.slice(s![row_i, bitrange.clone()]);
                    Self::map_array_to_int(&mut row[col_i], bits);
                }
            }
        }

        Ok(data)
    }
}

impl Source<String, usize, Integer> for VcdLoader {}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_subset() {
        const FILENAME: &'static str = "examples/verilator.vcd";

        let file = File::open(Path::new(FILENAME)).unwrap();
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header().unwrap();
        let timescale = header
            .timescale
            .map(|(n, ts)| VcdLoader::timescale_to_simtime(n, ts))
            .unwrap_or(SimTime::from_ps(1));
        let (_signals, info, namemap) = VcdLoader::load_all_scopes(&header);

        let ids = vec![
            "top.clk".to_string(),
            "top.hello.random".to_string()
        ];
        let record_ids: Vec<vcd::IdCode> = ids.iter()
            .filter_map(|id| {
                namemap.get(id)
            })
            .map(|x| x.clone())
            .collect();
        let record_cycles = 0..50;
        let cycle_time = SimTime::from_ps(1);

        let subset = VcdLoader::load_subset(&mut parser,&info, cycle_time, timescale, &record_ids,
            record_cycles).unwrap();

        println!("subset:\n{:?}", subset.data);

        assert_eq!(Value::X, subset.data[[0, 0]]);
        assert_eq!(Value::V1, subset.data[[1, 0]]);
        assert_eq!(Value::V0, subset.data[[2, 0]]);
        assert_eq!(Value::V1, subset.data[[3, 0]]);

        assert_eq!(Value::X, subset.data[[0, 1]]);
        for i in 1..40 {
            assert_eq!(Value::V1, subset.data[[i, 1]]);
        }
        assert_eq!(Value::V0, subset.data[[41, 1]]);
    }

    #[test]
    fn test_load_subset2() {
        const FILENAME: &'static str = "examples/core.vcd";

        let file = File::open(Path::new(FILENAME)).unwrap();
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header().unwrap();
        let timescale = header
            .timescale
            .map(|(n, ts)| VcdLoader::timescale_to_simtime(n, ts))
            .unwrap_or(SimTime::from_ps(1));
        let (_signals, info, namemap) = VcdLoader::load_all_scopes(&header);

        let ids = vec![
            "tb_core.clk".to_string(),
            "tb_core.reset".to_string(),
            "tb_core.uut.ifu.i0_pass_q[0:1]".to_string(),
        ];
        let record_ids: Vec<vcd::IdCode> = ids.iter()
            .filter_map(|id| {
                namemap.get(id)
            })
            .map(|x| x.clone())
            .collect();
        let record_cycles = 0..100;
        let cycle_time = SimTime::from_ps(100);

        let subset = VcdLoader::load_subset(&mut parser,&info, cycle_time, timescale, &record_ids,
            record_cycles).unwrap();

        println!("subset:\n{:?}", subset.data);

        assert_eq!(Value::V0, subset.data[[0, 0]]);
        assert_eq!(Value::V1, subset.data[[1, 0]]);
        assert_eq!(Value::V0, subset.data[[2, 0]]);
        assert_eq!(Value::V1, subset.data[[3, 0]]);

        for i in 0..9 {
            assert_eq!(Value::V1, subset.data[[i, 1]]);
        }
        assert_eq!(Value::V0, subset.data[[10, 1]]);

        assert_eq!(Value::X, subset.data[[0, 2]]);
        assert_eq!(Value::X, subset.data[[0, 3]]);

        assert_eq!(Value::V0, subset.data[[1, 2]]);
        assert_eq!(Value::V0, subset.data[[1, 3]]);
        
        assert_eq!(Value::V1, subset.data[[11, 2]]);
        assert_eq!(Value::V0, subset.data[[11, 3]]);
    }
}
