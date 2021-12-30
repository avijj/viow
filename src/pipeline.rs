use crate::error::*;
use crate::data::*;
use crate::formatting::WaveFormat;

//pub struct GenericPipeline<S, I, SrcId, PipeId, ExitId, SrcVal, PipeVal, ExitVal, A, B>
pub struct GenericPipeline<S, I, PipeId, ExitId, PipeVal, A, B>
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
    stages: Vec<Box<dyn Filter<I, PipeId, PipeVal, PipeVal>>>,
    exit_adapter: B,

    _mark: std::marker::PhantomData<*const ExitId>,
}

pub type Pipeline<S> = GenericPipeline<
    S,
    Vec<String>,
    usize,
    String,
    rug::Integer,
    SourceAdapter,
    ExitAdapter,
>;

impl<S, IPipe, PipeId, ExitId, PipeVal, A, B>
    GenericPipeline<S, IPipe, PipeId, ExitId, PipeVal, A, B>
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
impl<S, I, PipeId, ExitId, PipeVal, A, B> QuerySource<I>
    for GenericPipeline<S, I, PipeId, ExitId, PipeVal, A, B>
where
    I: IntoIterator<Item = ExitId>,
    //S: Source<I, SrcId, SrcVal>,
    //A: Transform<InValue = SrcVal, OutValue = PipeVal>
        //+ AssignId<FromId = SrcId, ToId = PipeId>
        //+ LookupId<FromId = PipeId, ToId = SrcId>,
    //B: AssignId<FromId = PipeId, ToId = ExitId> + LookupId<FromId = ExitId, ToId = PipeId>,
{
    type Id = ExitId;

    fn query_ids(&self) -> Result<I> {
        unimplemented!();
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        unimplemented!();
        //self.source.query_time()
    }

    fn query_format(&self, id: &Self::Id) -> Result<WaveFormat> {
        unimplemented!();
    }
}


impl<S, I, PipeId, ExitId, PipeVal, ExitVal, A, B> Sample
    for GenericPipeline<S, I, PipeId, ExitId, PipeVal, A, B>
where
    B: Transform<InValue = PipeVal, OutValue = ExitVal>
{
    type Id = ExitId;
    type Value = ExitVal;

    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
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
    type FromId = String;
    type ToId = usize;

    fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}

impl LookupId for SourceAdapter {
    type FromId = usize;
    type ToId = String;

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
    type FromId = usize;
    type ToId = String;

    fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}

impl LookupId for ExitAdapter {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId> {
        unimplemented!();
    }
}
