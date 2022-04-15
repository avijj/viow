use super::*;

use lru::LruCache;
use std::ops::Range;

#[derive(PartialEq, Eq, Hash, Debug)]
struct CacheIndex {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct CacheTile {
    data: Array2<Integer>,
}

pub(super) struct Cache {
    cache: LruCache<CacheIndex, CacheTile>,
    signals_per_tile: usize,
    cycles_per_tile: usize,
    num_signals: usize,
    num_cycles: usize,
}

impl Cache {
    pub(super) fn new(
        capacity: usize,
        signals_per_tile: usize,
        cycles_per_tile: usize,
        num_signals: usize,
        num_cycles: usize
    ) -> Self {
        Self {
            cache: LruCache::new(capacity),
            signals_per_tile,
            cycles_per_tile,
            num_signals,
            num_cycles,
        }
    }

    fn tile_index(&self, id: usize, cycle: usize) -> CacheIndex {
        CacheIndex {
            x: id / self.signals_per_tile,
            y: cycle / self.cycles_per_tile,
        }
    }

    fn tile_offset(&self, id: usize, cycle: usize) -> CacheIndex {
        CacheIndex {
            x: id % self.signals_per_tile,
            y: cycle % self.cycles_per_tile,
        }
    }

    fn load_tile(&self, pipe: &mut PipelineCId, index: &CacheIndex) -> CacheTile {
        let start_cycle = index.y * self.cycles_per_tile;
        let end_cycle = std::cmp::min(
            (index.y + 1) * self.cycles_per_tile,
            self.num_cycles
        );

        let a = pipe.query_time(start_cycle);
        let b = pipe.query_time(end_cycle);

        let start_id = index.x * self.signals_per_tile;
        let end_id = std::cmp::min(
            (index.x + 1) * self.signals_per_tile,
            self.num_signals
        );

        let ids = (start_id..end_id).collect();
        let data = pipe.sample(&ids, &SimTimeRange(a, b))
            .expect("Can't fill cache miss from pipe");

        CacheTile {
            data
        }
    }

    pub(super) fn get(&mut self, pipe: &mut PipelineCId, id: usize, cycle_range: Range<usize>) -> Array1<Integer> {
        debug_assert!(id < self.num_signals);
        debug_assert!(cycle_range.end <= self.num_cycles);

        let mut cur_cycle = cycle_range.start;
        let mut rv = Array1::default(cycle_range.end - cycle_range.start);

        //println!("rv: {:?}", rv);

        while cur_cycle < cycle_range.end {
            // cache index and offset within tile
            let tile_index = self.tile_index(id, cur_cycle);
            let tile_offset = self.tile_offset(id, cur_cycle);
            // start and end in return buffer
            let start_cycle = cur_cycle - cycle_range.start;
            let end_cycle = std::cmp::min(
                // end of tile
                (tile_index.y + 1) * self.cycles_per_tile,
                // end of requested data
                cycle_range.end
                //cycle_range.end - cycle_range.start,
                //cur_cycle + self.cycles_per_tile - cycle_range.start,
            ) - cycle_range.start;
            // end cycle within retrieved cache tile
            let tile_end_cycle = tile_offset.y + ((end_cycle + cycle_range.start) - cur_cycle);

            //println!("Cache::get(id: {}, cycle_range: {:?}):\n\tcur_cycle: {}\n\ttile_index: {:?}\n\ttile_offset: {:?}\n\tstart_cycle: {}\n\tend_cycle: {}\n\ttile_end_cycle: {}",
                //id, cycle_range, cur_cycle, tile_index, tile_offset, start_cycle, end_cycle, tile_end_cycle);

            if let Some(tile) = self.cache.get(&tile_index) {
                // cache hit
                //println!("Found tile: {:?}", tile);
                rv.slice_mut(s![start_cycle..end_cycle]).assign(
                    &tile
                        .data
                        .slice(s![tile_offset.y..tile_end_cycle, tile_offset.x]),
                );
            } else {
                // cache miss
                let tile = self.load_tile(pipe, &tile_index);

                //println!("Loaded tile: {:?}", tile);

                rv.slice_mut(s![start_cycle..end_cycle]).assign(
                    &tile
                        .data
                        .slice(s![tile_offset.y..tile_end_cycle, tile_offset.x]),
                );

                self.cache.put(tile_index, tile);
                //println!("Now at {} / {} entries", self.cache.len(), self.cache.cap());
            }

            cur_cycle += end_cycle - start_cycle;
        }

        rv
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::load::vcd::VcdLoader;


    #[test]
    fn test_wave_cache() {
        const CAPACITY: usize = 11;
        const SIG_PER_TILE: usize = 3;
        const CYC_PER_TILE: usize = 23;

        let loader = Box::new(VcdLoader::new("examples/verilator.vcd", Some(SimTime::from_ps(1))).unwrap());
        let mut pipe = PipelineCId::new(loader).unwrap();
        pipe.query_init().unwrap();
        let num_signals = pipe.query_signals().unwrap().len();
        let num_cycles = pipe.query_cycle_count();
        let mut cache = Cache::new(CAPACITY, SIG_PER_TILE, CYC_PER_TILE, num_signals, num_cycles);

        let needle = cache.get(&mut pipe, 7, 0..50);

        assert_eq!(Integer::from(0), needle[0]);
        for i in 1..40 {
            assert_eq!(Integer::from(1), needle[i]);
        }
        assert_eq!(Integer::from(0), needle[41]);

        assert_eq!(Integer::from(2), cache.get(&mut pipe, 5, 0..20)[13]);
        assert_eq!(Integer::from(3), cache.get(&mut pipe, 5, 0..16)[15]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 0..24)[23]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 10..24)[13]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 23..24)[0]);

        for i in 0..15 {
            cache.get(&mut pipe, i, 0..200);
        }

        assert_eq!(Integer::from(2), cache.get(&mut pipe, 5, 0..20)[13]);
        assert_eq!(Integer::from(3), cache.get(&mut pipe, 5, 0..16)[15]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 0..24)[23]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 10..24)[13]);
        assert_eq!(Integer::from(7), cache.get(&mut pipe, 5, 23..24)[0]);
    }
}
