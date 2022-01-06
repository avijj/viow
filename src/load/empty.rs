use super::*;
use crate::data::*;
use ndarray::prelude::*;

pub struct EmptyLoader {}

impl EmptyLoader {
    pub fn new() -> Self {
        Self {}
    }
}


impl QuerySource for EmptyLoader {
    type Id = String;
    type IntoSignalIter = Vec<Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter> {
        Ok(vec![])
    }

    fn query_time(&self) -> Result<SimTimeRange> {
        let start = SimTime::zero();
        let stop = SimTime::zero();

        Ok(SimTimeRange(start, stop))
    }
}

impl LookupId for EmptyLoader {
    type FromId = String;
    type ToId = usize;

    fn lookup_id(&self, id: &Self::FromId) -> Result<Self::ToId> {
        Err(Error::NotFound(id.clone()))
    }

    fn rev_lookup_id(&self, id: &Self::ToId) -> Result<Self::FromId> {
        Err(Error::IdOutOfRange(*id, 0))
    }
}

impl Sample for EmptyLoader {
    type Id = String;
    type Value = Integer;

    fn sample(&self, _ids: &Vec<Self::Id>, _times: &SimTimeRange) -> Result<CycleValues<Self::Value>> {
        let data = Array2::default((0, 0));
        Ok(data)
    }
}

impl Source<String, usize, Integer> for EmptyLoader {}
