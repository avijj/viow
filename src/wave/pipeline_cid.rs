use crate::data::*;
use crate::error::*;
use crate::pipeline::*;

/// Pipeline adapter providing contiguous signal ids.
pub(super) struct PipelineCId {
    /// Underlying Pipeline that is adapted.
    pipe: Pipeline,

    /// Maps a contiguous range of ids to a potentially sparse set
    ///
    /// Index is id returned by query_source. Stored value at each position is the matching id in
    /// the underlying pipe.
    idmap: Vec<usize>,
}

impl PipelineCId {
    pub(super) fn new(source: SrcBox) -> Result<Self> {
        let pipe = Pipeline::new(source);
        let idmap = Vec::new();

        Ok(Self { pipe, idmap })
    }

    pub(super) fn push(self, stage: FilterBox) -> Self {
        Self {
            pipe: self.pipe.push(stage),
            ..self
        }
    }

    pub(super) fn pop(self) -> (Self, Option<FilterBox>) {
        let (pipe, tail) = self.pipe.pop();
        let rv = Self { pipe, ..self };

        (rv, tail)
    }
}

impl QuerySource for PipelineCId {
    type Id = usize;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_init(&mut self) -> Result<()> {
        let idmap = self.pipe.query_signals()?
            .into_iter()
            .map(|signal| signal.id)
            .collect();

        self.idmap = idmap;
        Ok(())
    }

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        Ok(
            self.pipe.query_signals()?
                .into_iter()
                .enumerate()
                .map(|(i, signal)| {
                    Signal {
                        id: i,
                        ..signal
                    }
                })
                .collect()
        )
    }

    fn query_time_range(&self) -> Result<SimTimeRange> {
        self.pipe.query_time_range()
    }

    fn query_time(&self, cycle: usize) -> SimTime {
        self.pipe.query_time(cycle)
    }

    fn query_cycle_count(&self) -> usize {
        self.pipe.query_cycle_count()
    }
}

impl Sample for PipelineCId {
    type Id = usize;
    type Value = rug::Integer;

    fn sample(
        &mut self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        let mapped_ids: Vec<usize> = ids.iter()
            .filter_map(|id| self.idmap.get(*id))
            .map(|i| *i)
            .collect();
        self.pipe.sample(&mapped_ids, times)
    }
}

impl ConfigurePipeline for PipelineCId {
    fn configure_pipeline(&mut self, config: &PipelineConfig) -> Result<()> {
        self.pipe.configure_pipeline(config)
    }
}
