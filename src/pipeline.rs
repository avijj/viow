use crate::data::*;
use crate::error::*;
use crate::formatting::WaveFormat;

pub struct GenericPipeline<S, PipeId, ExitId, PipeVal, A, B> {
    //source: S,
    source_adapter: A,  // FIXME remove
    stages: Stage<S, PipeId, PipeVal>,
    //stages:
        //Vec<Box<dyn Filter<PipeId, PipeVal, PipeVal, IntoSignalIter = Vec<(PipeId, WaveFormat)>>>>,
    exit_adapter: B,

    _mark: std::marker::PhantomData<*const ExitId>,
}

pub type Pipeline<S> = GenericPipeline<S, usize, String, rug::Integer, SourceAdapter, ExitAdapter>;

impl<S, SrcId, PipeId, ExitId, SrcVal, PipeVal, A, B> GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    A: LookupId<FromId = SrcId, ToId = PipeId> + Transform<InValue = SrcVal, OutValue = PipeVal>,
    S: Source<SrcId, SrcVal>
{
    pub fn new(source: S, source_adapter: A, exit_adapter: B) -> Self {
        let source_stage = Stage::Src(source);

        Self {
            source_adapter,
            stages: source_stage,
            exit_adapter,
            _mark: std::marker::PhantomData,
        }
    }
}

impl<S, SrcId, PipeId, ExitId, SrcVal, PipeVal, A, B> QuerySource
    for GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    A: LookupId<FromId = SrcId, ToId = PipeId> + Transform<InValue = SrcVal, OutValue = PipeVal>,
    B: LookupId<FromId = PipeId, ToId = ExitId>,
    S: Source<SrcId, SrcVal> + LookupId<FromId = SrcId, ToId = PipeId>,
{
    type Id = ExitId;
    type IntoSignalIter = Vec<(Self::Id, WaveFormat)>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let rv: Result<Vec<(ExitId, WaveFormat)>>;
        rv = self.stages
            .query_signals()?
            .into_iter()
            .map(|(pipe_id, format)| -> Result<_> {
                let exit_id = self.exit_adapter.lookup_id(pipe_id)?;
                Ok((exit_id, format))
            })
            .collect();

        rv
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        self.stages.query_time()
    }
}

impl<S, PipeId, ExitId, PipeVal, ExitVal, A, B> Sample
    for GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    B: Transform<InValue = PipeVal, OutValue = ExitVal>,
{
    type Id = ExitId;
    type Value = ExitVal;

    fn sample(
        &self,
        ids: &Vec<Self::Id>,
        times: &SimTimeRange,
    ) -> Result<CycleValues<Self::Value>> {
        unimplemented!();
    }
}

//
// SourceAdapter
//

pub struct SourceAdapter {}

impl Transform for SourceAdapter {
    type InValue = rug::Integer;
    type OutValue = rug::Integer;

    fn transform(&self, value: Self::InValue) -> Self::OutValue {
        value
    }
}

impl AssignId for SourceAdapter {
    type FromId = usize;
    type ToId = String;

    fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}

impl LookupId for SourceAdapter {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}


// //
// // SourceStage
// //
// 
// // FIXME reduce to concrete types, otherwise not really implementable.
// pub struct SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> 
// {
//     source: S,
// 
//     _mark_0: std::marker::PhantomData<SrcId>,
//     _mark_1: std::marker::PhantomData<PipeId>,
//     _mark_2: std::marker::PhantomData<SrcVal>,
//     _mark_3: std::marker::PhantomData<PipeVal>,
// }
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> {
//     fn new(source: S) -> Self {
//         Self {
//             source,
//             _mark_0: std::marker::PhantomData,
//             _mark_1: std::marker::PhantomData,
//             _mark_2: std::marker::PhantomData,
//             _mark_3: std::marker::PhantomData,
//         }
//     }
// }
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> Transform for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> {
//     type InValue = PipeVal;
//     type OutValue = PipeVal;
// 
//     fn transform(&self, value: Self::InValue) -> Self::OutValue {
//         unimplemented!();
//     }
// }
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> QuerySource for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal>
// where
//     S: Source<SrcId, SrcVal>
// {
//     type Id = PipeId;
//     type IntoSignalIter = Vec<(Self::Id, WaveFormat)>;
// 
//     fn query_signals(&self) -> Result<Self::IntoSignalIter> {
//         let src_sigs = self.source.query_signals()?;
//         let pipe_sigs: Result<Vec<_>> = src_sigs.into_iter()
//             .map(|(src_id, format)| -> Result<_> {
//                 let pipe_id = self.lookup_id(src_id)?;
//                 Ok((pipe_id, format))
//             })
//             .collect();
// 
//         pipe_sigs
//     }
// 
//     fn query_time(&self) -> Result<SimTimeRange> {
//         self.source.query_time()
//     }
// }
// 
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> TranslateSignal for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> {
//     type InId = PipeId;
//     type OutId = PipeId;
// 
//     fn translate_signal(&self, id: &Self::InId) -> Result<Self::OutId> {
//         unimplemented!();
//     }
// }
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> LookupId for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> {
//     type FromId = SrcId;
//     type ToId = PipeId;
// 
//     fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId> {
//         unimplemented!();
//     }
// }
// 
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> Sample for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal> {
//     type Id = PipeId;
//     type Value = PipeVal;
// 
//     fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
//         unimplemented!();
//     }
// }
// 
// impl<S, SrcId, PipeId, SrcVal, PipeVal> Filter<PipeId, PipeVal, PipeVal> for SourceStage<S, SrcId, PipeId, SrcVal, PipeVal>
// where
//     S: Source<SrcId, SrcVal>,
// {}


//
// ExitAdapter
//

pub struct ExitAdapter {}

impl Transform for ExitAdapter {
    type InValue = rug::Integer;
    type OutValue = rug::Integer;

    fn transform(&self, value: Self::InValue) -> Self::OutValue {
        value
    }
}

impl AssignId for ExitAdapter {
    type FromId = String;
    type ToId = usize;

    fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}

impl LookupId for ExitAdapter {
    type FromId = usize;
    type ToId = String;

    fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}


//
// Pipeline stages
//

//enum StagePtr<S: Sized, PipeId, PipeVal> {
    //Src(S),
    //Fil(Box<Stage<S, PipeId, PipeVal>>)
//}

//pub struct Stage<S: Sized, PipeId, PipeVal> {
    //filter: Box<dyn Filter<PipeId, PipeVal, PipeVal, IntoSignalIter = Vec<(PipeId, WaveFormat)>>>,
    ////prev: Option<Box<Self>>
    //prev: StagePtr<S, PipeId, PipeVal>
//}

pub enum Stage<S, PipeId, PipeVal> {
    Src(S),
    Fil(Box<Stage<S, PipeId, PipeVal>>, Box<dyn Filter<PipeId, PipeVal, PipeVal, IntoSignalIter = Vec<(PipeId, WaveFormat)>>>)
}

impl<S, SrcId, PipeId, PipeVal> QuerySource for Stage<S, PipeId, PipeVal>
    where
        S: QuerySource<Id = SrcId> + LookupId<FromId = SrcId, ToId = PipeId>
{
    type Id = PipeId;
    type IntoSignalIter = Vec<(Self::Id, WaveFormat)>;

    // FIXME how to solve recursion to source
    // The approache with SourceStage as Filter is shit. It requires tons of impls and useless
    // boilerplate code. Instead, I can turn `prev` into an enum with three(two?) variants: (None),
    // Filter-stage, Source. Then source would get a special code path in recursion functions and
    // only needs to implement a minimal set of traits instead of full Filter.
    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let mut translated;

        match self {
            Self::Fil(ref prev, ref filter) => {
                let prev_signals = prev.query_signals()?;
                let translated_res: Result<Vec<_>> = prev_signals.iter()
                    .map(|(signal, format)| Ok((filter.translate_signal(signal)?, format.clone())))
                    .collect();
                translated = translated_res?;
                
                let mut src_signals = filter.query_signals()?;
                translated.append(&mut src_signals);
            }

            Self::Src(ref src) => {
                let src_signals = src.query_signals()?;
                let translated_res: Result<Vec<_>> = src_signals.into_iter()
                    .map(|(signal, format)| Ok((src.lookup_id(signal)?, format.clone())))
                    .collect();
                translated = translated_res?;
            }
        }

        Ok(translated)
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        unimplemented!();
    }
}

