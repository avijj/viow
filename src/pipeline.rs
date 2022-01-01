use crate::data::*;
use crate::error::*;

pub type Pipeline<S> = Stage<S, usize, rug::Integer>;

//
// Pipeline stages
//

pub enum Stage<S, PipeId, PipeVal> {
    Src(S),
    Fil(
        Box<Stage<S, PipeId, PipeVal>>,
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

impl<S, PipeId, PipeVal> Stage<S, PipeId, PipeVal> {
    pub fn new(source: S) -> Self {
        Self::Src(source)
    }
}

impl<S, SrcId, PipeId, PipeVal> QuerySource for Stage<S, PipeId, PipeVal>
where
    S: QuerySource<Id = SrcId> + LookupId<FromId = SrcId, ToId = PipeId>,
{
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

    fn query_time(&self) -> Result<SimTimeRange> {
        match self {
            Self::Fil(ref prev, _) => prev.query_time(),

            Self::Src(ref src) => src.query_time(),
        }
    }
}

impl<S, SrcId, PipeId, PipeVal> Sample for Stage<S, PipeId, PipeVal>
where
    PipeId: Clone,
    S: LookupId<FromId = SrcId, ToId = PipeId> + Sample<Id = SrcId, Value = PipeVal>,
{
    type Id = PipeId;
    type Value = PipeVal;

    fn sample(
        &self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        match self {
            Self::Fil(ref prev, ref filter) => {
                let trans_ids = filter.rev_translate_ids(ids.to_vec())?;
                let mut vals = prev.sample(&trans_ids, times)?;
                for elem in vals.iter_mut() {
                    filter.transform(elem);
                }

                Ok(vals)
            }

            Self::Src(ref src) => {
                let src_ids: Result<Vec<_>> = ids.iter().map(|id| src.rev_lookup_id(id)).collect();
                let src_ids = src_ids?;
                let src_vals = src.sample(&src_ids, times)?;

                Ok(src_vals)
            }
        }
    }
}

