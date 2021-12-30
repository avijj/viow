pub mod transforms;
mod simtime;

pub use simtime::*;

use ndarray::prelude::*;

use crate::error::*;
use crate::formatting::WaveFormat;

//
// Types
//



pub type CycleValues<T> = Array2<T>;

//
// Traits
//

pub trait QuerySource {
    type Id;
    type IntoSignalIter: IntoIterator<Item = (Self::Id, WaveFormat)>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter>;
    fn query_time(&self) -> Result<SimTimeRange>;
}

pub trait AssignId {
    type FromId;
    type ToId;

    fn assign_id(&mut self, id: Self::FromId) -> Result<Self::ToId>;
}

pub trait LookupId {
    type FromId;
    type ToId;

    fn lookup_id(&self, id: Self::FromId) -> Result<Self::ToId>;
}

pub trait Sample {
    type Id;
    type Value;

    //fn sample(
    //&self,
    //ids: impl IntoIterator<Item = &'a Self::Id>,
    //times: impl AsRef<SimTimeRange>,
    //) -> Result<CycleValues<Self::Value>>;

    // Need concrete types as arguments, because of use as trait object.
    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange)
        -> Result<CycleValues<Self::Value>>;
}

pub trait Transform {
    type InValue;
    type OutValue;

    fn transform(&self, value: Self::InValue) -> Self::OutValue;
}

pub trait Source<Id, V>: QuerySource<Id = Id> + Sample<Id = Id, Value = V> {}

pub trait Filter<Id, InVal, OutVal>:
    QuerySource<Id = Id>
    + Sample<Id = Id, Value = OutVal>
    + Transform<InValue = InVal, OutValue = OutVal>
{
}
