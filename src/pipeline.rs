pub mod filter;

use crate::data::*;
use crate::error::*;

pub type Pipeline = Stage<String, usize, rug::Integer>;
pub type SrcBox = Box<dyn Source<String, usize, rug::Integer, IntoSignalIter = Vec<Signal<String>>>>;
pub type FilterBox = Box<dyn Filter<usize, rug::Integer, IntoSigIter = Vec<Signal<usize>>, IntoIdIter = Vec<usize>>>;

//
// Pipeline stages
//

pub enum Stage<SrcId, PipeId, PipeVal> {
    Src(Box<dyn Source<SrcId, PipeId, PipeVal, IntoSignalIter = Vec<Signal<SrcId>>>>),
    Fil(
        Box<Stage<SrcId, PipeId, PipeVal>>,
        Box<
            dyn Filter<
                PipeId,
                PipeVal,
                IntoSigIter = Vec<Signal<PipeId>>,
                IntoIdIter = Vec<PipeId>,
            >,
        >,
    ),
}

impl<SrcId, PipeId, PipeVal> Stage<SrcId, PipeId, PipeVal> {
    pub fn new(source: Box<dyn Source<SrcId, PipeId, PipeVal, IntoSignalIter = Vec<Signal<SrcId>>>>) -> Self {
        Self::Src(source)
    }

    pub fn push(self, stage: Box<dyn Filter< PipeId, PipeVal, IntoSigIter = Vec<Signal<PipeId>>, IntoIdIter = Vec<PipeId> >>) -> Self {
        Self::Fil(Box::new(self), stage)
    }

    pub fn pop(self) -> (Self, Option<Box<dyn Filter< PipeId, PipeVal, IntoSigIter = Vec<Signal<PipeId>>, IntoIdIter = Vec<PipeId> >>>) {
        match self {
            Self::Fil(prev, filter) => {
                (*prev, Some(filter))
            }

            Self::Src(_) => {
                (self, None)
            }
        }
    }
}

impl<SrcId, PipeId, PipeVal> QuerySource for Stage<SrcId, PipeId, PipeVal> {
    type Id = PipeId;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    // how to solve recursion to source?
    // The approache with SourceStage as Filter is shit. It requires tons of impls and useless
    // boilerplate code. Instead, I can turn `prev` into an enum with three(two?) variants: (None),
    // Filter-stage, Source. Then source would get a special code path in recursion functions and
    // only needs to implement a minimal set of traits instead of full Filter.
    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let translated;

        match self {
            Self::Fil(ref prev, ref filter) => {
                let prev_signals = prev.query_signals()?;
                translated = filter.translate_signals(prev_signals)?;

                // not used right now, would allow filters to create their own signals.
                //let mut src_signals = filter.query_signals()?;
                //translated.append(&mut src_signals);
            }

            Self::Src(ref src) => {
                let src_signals = src.query_signals()?;
                let translated_res: Result<Vec<_>> = src_signals
                    .into_iter()
                    .map(|signal| {
                        Ok(Signal {
                            id: src.lookup_id(&signal.id)?,
                            name: signal.name,
                            format: signal.format,
                        })
                    })
                    .collect();
                translated = translated_res?;
            }
        }

        Ok(translated)
    }

    fn query_time_range(&self) -> Result<SimTimeRange> {
        match self {
            Self::Fil(ref prev, _) => prev.query_time_range(),

            Self::Src(ref src) => src.query_time_range(),
        }
    }

    fn query_time(&self, cycle: usize) -> SimTime {
        match self {
            Self::Fil(ref prev, _) => prev.query_time(cycle),
            Self::Src(ref src) => src.query_time(cycle),
        }
    }

    fn query_cycle_count(&self) -> usize {
        match self {
            Self::Fil(ref prev, _) => prev.query_cycle_count(),
            Self::Src(ref src) => src.query_cycle_count(),
        }
    }
}

impl<SrcId, PipeId, PipeVal> Sample for Stage<SrcId, PipeId, PipeVal>
where
    PipeId: Clone,
{
    type Id = PipeId;
    type Value = PipeVal;

    fn sample(
        &mut self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        match self {
            Self::Fil(ref mut prev, ref mut filter) => {
                let trans_ids = filter.rev_translate_ids(ids.to_vec())?;
                let mut vals = prev.sample(&trans_ids, times)?;
                filter.transform(&mut vals);

                Ok(vals)
            }

            Self::Src(ref mut src) => {
                let src_ids: Result<Vec<_>> = ids.iter().map(|id| src.rev_lookup_id(id)).collect();
                let src_ids = src_ids?;
                let src_vals = src.sample(&src_ids, times)?;

                Ok(src_vals)
            }
        }
    }
}

impl <SrcId, PipeId, PipeVal> ConfigurePipeline for Stage<SrcId, PipeId, PipeVal> {
    fn configure_pipeline(&mut self, config: &PipelineConfig) -> Result<()> {
        match self {
            Self::Src(_) => {
                Ok(())
            }

            Self::Fil(prev, filter) => {
                filter.configure_pipeline(config)?;
                prev.configure_pipeline(config)
            }
        }
    }
}
