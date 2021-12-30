pub mod transforms;

use ndarray::prelude::*;

use crate::error::*;
use crate::formatting::WaveFormat;

//
// Types
//

#[derive(Debug)]
pub enum SimTime {
    Fs(u64),
    Ps(u64),
    Us(u64),
    Ns(u64),
    Ms(u64),
    S(u64),
}

#[derive(Debug)]
pub struct SimTimeRange(SimTime, SimTime);

pub type CycleValues<T> = Array2<T>;

//
// Traits
//

pub trait QuerySource<IntoIdIter>
where
    IntoIdIter: IntoIterator<Item = Self::Id>,
{
    type Id;

    fn query_ids(&self) -> Result<IntoIdIter>;
    fn query_time(&self) -> Result<SimTimeRange>;
    fn query_format(&self, id: &Self::Id) -> Result<WaveFormat>;
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

pub trait Sample
{
    type Id;
    type Value;

    //fn sample(
        //&self,
        //ids: impl IntoIterator<Item = &'a Self::Id>,
        //times: impl AsRef<SimTimeRange>,
    //) -> Result<CycleValues<Self::Value>>;
    
    // Need concrete types as arguments, because of use as trait object.
    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange) -> Result<CycleValues<Self::Value>>;
}

pub trait Transform {
    type InValue;
    type OutValue;

    fn transform(&self, value: Self::InValue) -> Self::OutValue;
}

pub trait Source<I, Id, V>: QuerySource<I, Id = Id> + Sample<Id = Id, Value = V>
where
    I: IntoIterator<Item = <Self as QuerySource<I>>::Id>,
{
}

pub trait Filter<I, Id, InVal, OutVal>:
    QuerySource<I, Id = Id> + Sample<Id = Id, Value = OutVal> + Transform<InValue = InVal, OutValue = OutVal>
where
    I: IntoIterator<Item = <Self as QuerySource<I>>::Id>
{
}
