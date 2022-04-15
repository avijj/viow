use crate::data::*;
use crate::error::*;
use crate::pipeline::*;

/// Pipeline adapter providing contiguous signal ids.
pub(super) struct PipelineCId {
    pipe: Pipeline,
}

impl PipelineCId {
    pub(super) fn new(source: SrcBox) -> Self {
        let pipe = Pipeline::new(source);

        Self { pipe }
    }

    pub(super) fn push(self, stage: FilterBox) -> Self {
        Self {
            pipe: self.pipe.push(stage),
        }
    }

    pub(super) fn pop(self) -> (Self, Option<FilterBox>) {
        let (pipe, tail) = self.pipe.pop();
        let rv = Self { pipe };

        (rv, tail)
    }
}

impl QuerySource for PipelineCId {
    type Id = usize;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        self.pipe.query_signals()
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
        self.pipe.sample(ids, times)
    }
}

impl ConfigurePipeline for PipelineCId {
    fn configure_pipeline(&mut self, config: &PipelineConfig) -> Result<()> {
        self.pipe.configure_pipeline(config)
    }
}
