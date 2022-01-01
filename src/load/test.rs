use super::*;

pub struct TestLoader {
    num_cycles: usize,
    num_signals: usize,
}

impl TestLoader {
    pub fn _new(num_signals: usize, num_cycles: usize) -> Self {
        Self {
            num_cycles,
            num_signals,
        }
    }
}


impl LoadDeclarations for TestLoader {
    fn load_declarations(&self) -> Result<Vec<SignalDeclaration>> {
        Ok(
            (0..self.num_signals).map(|i| SignalDeclaration {
                    name: format!("row_{}", i),
                    format: WaveFormat::Bit
                })
                .collect()
        )
    }
}


impl LoadLength for TestLoader {
    fn load_length(&self) -> Result<usize> {
        Ok(self.num_cycles)
    }
}


impl LoadWaveform for TestLoader {
    fn load_waveform(&self, _name: impl AsRef<str>, cycles: Range<usize>) -> Result<Vec<Integer>> {
        if cycles.contains(&self.num_cycles) {
            Ok(vec![Integer::from(0); (cycles.start .. self.num_cycles).len()])
        } else {
            Ok(vec![Integer::from(0); cycles.len()])
        }
    }
}
