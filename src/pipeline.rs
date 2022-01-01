use crate::data::*;
use crate::error::*;
use crate::formatting::WaveFormat;

// pub struct GenericPipeline<S, PipeId, SrcVal, PipeVal, E> {
//     stages: Stage<S, PipeId, PipeVal>,
// 
//     _mark_1: std::marker::PhantomData<*const SrcVal>,
// }
// 
// pub type Pipeline<S> = GenericPipeline<S, usize, String, rug::Integer, rug::Integer, ExitAdapter>;
// 
// impl<S, PipeId, ExitId, SrcVal, PipeVal, E> GenericPipeline<S, PipeId, ExitId, SrcVal, PipeVal, E> {
//     pub fn new(source: S, exit_adapter: E) -> Self {
//         let source_stage = Stage::Src(source);
// 
//         Self {
//             stages: source_stage,
//             exit_adapter,
//             _mark_0: std::marker::PhantomData,
//             _mark_1: std::marker::PhantomData,
//         }
//     }
// }
// 
// impl<S, SrcId, PipeId, ExitId, SrcVal, PipeVal, E> QuerySource
//     for GenericPipeline<S, PipeId, ExitId, SrcVal, PipeVal, E>
// where
//     E: LookupId<FromId = PipeId, ToId = ExitId>,
//     S: Source<SrcId, SrcVal> + LookupId<FromId = SrcId, ToId = PipeId>,
// {
//     type Id = ExitId;
//     type IntoSignalIter = Vec<Signal<Self::Id>>;
// 
//     fn query_signals(&self) -> Result<Self::IntoSignalIter> {
//         let rv: Result<Vec<Signal<ExitId>>>;
//         rv = self.stages
//             .query_signals()?
//             .into_iter()
//             .map(|signal| -> Result<_> {
//                 Ok(Signal {
//                     id: self.exit_adapter.lookup_id(signal.id)?,
//                     name: signal.name,
//                     format: signal.format
//                 })
//             })
//             .collect();
// 
//         rv
//     }
// 
//     fn query_time(&self) -> Result<SimTimeRange> {
//         self.stages.query_time()
//     }
// }
// 
// impl<S, PipeId, ExitId, SrcVal, PipeVal, ExitVal, E> Sample
//     for GenericPipeline<S, PipeId, ExitId, SrcVal, PipeVal, E>
// where
//     E: Transform<InValue = PipeVal, OutValue = ExitVal>,
// {
//     type Id = ExitId;
//     type Value = ExitVal;
// 
//     fn sample(
//         &self,
//         ids: &Vec<Self::Id>,
//         times: &SimTimeRange,
//     ) -> Result<CycleValues<Self::Value>> {
//         unimplemented!();
//     }
// }
// 
// 
// //
// // ExitAdapter
// //
// 
// pub struct ExitAdapter {}
// 
// impl Transform for ExitAdapter {
//     type InValue = rug::Integer;
//     type OutValue = rug::Integer;
// 
//     fn transform(&self, value: Self::InValue) -> Self::OutValue {
//         value
//     }
// }
// 
// impl AssignId for ExitAdapter {
//     type FromId = String;
//     type ToId = usize;
// 
//     fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId> {
//         unimplemented!();
//     }
// }
// 
// impl LookupId for ExitAdapter {
//     type FromId = usize;
//     type ToId = String;
// 
//     fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId> {
//         unimplemented!();
//     }
// }


pub type Pipeline<S> = Stage<S, usize, rug::Integer>;


//
// Pipeline stages
//

pub enum Stage<S, PipeId, PipeVal> {
    Src(S),
    Fil(Box<Stage<S, PipeId, PipeVal>>, Box<dyn Filter<PipeId, PipeVal, PipeVal, IntoSignalIter = Vec<Signal<PipeId>>>>)
}


impl<S, PipeId, PipeVal> Stage<S, PipeId, PipeVal> {
    pub fn new(source: S) -> Self {
        Self::Src(source)
    }
}


impl<S, SrcId, PipeId, PipeVal> QuerySource for Stage<S, PipeId, PipeVal>
    where
        S: QuerySource<Id = SrcId> + LookupId<FromId = SrcId, ToId = PipeId>
{
    type Id = PipeId;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    // how to solve recursion to source?
    // The approache with SourceStage as Filter is shit. It requires tons of impls and useless
    // boilerplate code. Instead, I can turn `prev` into an enum with three(two?) variants: (None),
    // Filter-stage, Source. Then source would get a special code path in recursion functions and
    // only needs to implement a minimal set of traits instead of full Filter.
    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let mut translated;

        match self {
            Self::Fil(ref prev, ref filter) => {
                let prev_signals = prev.query_signals()?;
                let translated_res: Result<Vec<_>> = prev_signals.into_iter()
                    .map(|signal| {
                        Ok( Signal {
                            id: filter.translate_signal(&signal.id)?,
                            ..signal
                        } )
                    })
                    .collect();
                translated = translated_res?;
                
                let mut src_signals = filter.query_signals()?;
                translated.append(&mut src_signals);
            }

            Self::Src(ref src) => {
                let src_signals = src.query_signals()?;
                let translated_res: Result<Vec<_>> = src_signals.into_iter()
                    .map(|signal| Ok(
                        Signal {
                            id: src.lookup_id(&signal.id)?,
                            name: signal.name,
                            format: signal.format
                        }))
                    .collect();
                translated = translated_res?;
            }
        }

        Ok(translated)
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        match self {
            Self::Fil(ref prev, _) => {
                prev.query_time()
            }

            Self::Src(ref src) => {
                src.query_time()
            }
        }
    }
}


impl<S, SrcId, PipeId, PipeVal> Sample for Stage<S, PipeId, PipeVal>
where
    S: LookupId<FromId = SrcId, ToId = PipeId> + Sample<Id = SrcId, Value = PipeVal>
{
    type Id = PipeId;
    type Value = PipeVal;

    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
        match self {
            Self::Fil(ref prev, ref filter) => {
                unimplemented!();
            }

            Self::Src(ref src) => {
                let src_ids: Result<Vec<_>> = ids.iter()
                    .map(|id| src.rev_lookup_id(id))
                    .collect();
                let src_ids = src_ids?;
                let src_vals = src.sample(&src_ids, times)?;

                Ok(src_vals)
            }
        }
    }
}



