use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;

//pub struct GenericPipeline<S, I, SrcId, PipeId, ExitId, SrcVal, PipeVal, ExitVal, A, B>
pub struct GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    //I: IntoIterator<Item = <S as QuerySource<I>>::Id>,
    //S: Source<I, SrcId, SrcVal>,
    //A: Transform<InValue = SrcVal, OutValue = PipeVal>
        //+ AssignId<FromId = SrcId, ToId = PipeId>
        //+ LookupId<FromId = PipeId, ToId = SrcId>,
    //B: Transform<InValue = PipeVal, OutValue = ExitVal>
        //+ AssignId<FromId = PipeId, ToId = ExitId>
        //+ LookupId<FromId = ExitId, ToId = PipeId>,
{
    source: S,
    source_adapter: A,
    stages: Vec<Box<dyn Filter<PipeId, PipeVal, PipeVal, IntoSignalIter = Vec<(PipeId, WaveFormat)>>>>,
    exit_adapter: B,

    _mark: std::marker::PhantomData<*const ExitId>,
}

pub type Pipeline<S> = GenericPipeline<
    S,
    usize,
    String,
    rug::Integer,
    SourceAdapter,
    ExitAdapter,
>;

impl<S, PipeId, ExitId, PipeVal, A, B>
    GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
{
    pub fn new(source: S, source_adapter: A, exit_adapter: B) -> Self {
        Self {
            source,
            source_adapter,
            stages: vec![],
            exit_adapter,
            _mark: std::marker::PhantomData,
        }
    }
}

//impl<S, I, SrcId, PipeId, ExitId, SrcVal, PipeVal, A, B> QuerySource<I>
impl<S, SrcId, PipeId, ExitId, SrcVal, PipeVal, A, B> QuerySource
    for GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    A: LookupId<FromId = SrcId, ToId = PipeId> + Transform<InValue = SrcVal, OutValue = PipeVal>,
    B: LookupId<FromId = PipeId, ToId = ExitId>,
    S: Source<SrcId, SrcVal>,
{
    type Id = ExitId;
    type IntoSignalIter = Vec<(Self::Id, WaveFormat)>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        let rv: Result<Vec<(ExitId, WaveFormat)>>;

        if let Some(end_stage) = self.stages.last() {
            // FIXME need to actually walk through the pipe
            rv = end_stage.query_signals()?.into_iter()
                .map(|(pipe_id, format)| -> Result<_> {
                    let exit_id = self.exit_adapter.lookup_id(pipe_id)?;
                    Ok((exit_id, format))
                })
                .collect();
        } else {
            rv = self.source.query_signals()?.into_iter()
                .map(|(src_id, format)| -> Result<_> {
                    let pipe_id = self.source_adapter.lookup_id(src_id)?;
                    let exit_id = self.exit_adapter.lookup_id(pipe_id)?;
                    Ok((exit_id, format))
                })
                .collect();
        }

        rv
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        if let Some(end_stage) = self.stages.last() {
            // FIXME need to actually walk through the pipe
            end_stage.query_time()
        } else {
            self.source.query_time()
        }
    }
}


impl<S, PipeId, ExitId, PipeVal, ExitVal, A, B> Sample
    for GenericPipeline<S, PipeId, ExitId, PipeVal, A, B>
where
    B: Transform<InValue = PipeVal, OutValue = ExitVal>
{
    type Id = ExitId;
    type Value = ExitVal;

    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
        if let Some(end_stage) = self.stages.last() {
            // FIXME need to actually walk through the pipe
            unimplemented!();
        } else {
            // FIXME translate ids backward to source
        }

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


//
// SourceAdapter
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
