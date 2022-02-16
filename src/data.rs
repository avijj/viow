mod simtime;

pub use simtime::*;

use ndarray::prelude::*;

use crate::error::*;
use crate::formatting::WaveFormat;

//
// Types
//

pub struct Signal<I> {
    pub id: I,
    pub name: String,
    pub format: WaveFormat,
}

pub type CycleValues<T> = Array2<T>;

#[derive(Default)]
pub struct PipelineConfig {
    pub name_list: Vec<String>,
    pub enable_filter_list: bool,
}

//
// Traits
//

pub trait QuerySource {
    type Id;
    type IntoSignalIter: IntoIterator<Item = Signal<Self::Id>>;

    fn query_signals(&self) -> Result<Self::IntoSignalIter>;
    fn query_time(&self) -> Result<SimTimeRange>;
}

pub trait LookupId {
    type FromId;
    type ToId;

    fn lookup_id(&self, id: &Self::FromId) -> Result<Self::ToId>;
    fn rev_lookup_id(&self, id: &Self::ToId) -> Result<Self::FromId>;
}

pub trait Sample {
    type Id;
    type Value;

    // Need concrete types as arguments, because of use as trait object.
    fn sample(&self, ids: &Vec<Self::Id>, times: &SimTimeRange)
        -> Result<CycleValues<Self::Value>>;
}

pub trait Transform {
    type Value;

    fn transform(&self, _value: &mut Self::Value) {}
}

pub trait TranslateSignals<I> {
    type IntoSigIter: IntoIterator<Item = Signal<I>>;
    type IntoIdIter: IntoIterator<Item = I>;

    fn translate_signals(&self, signals: Self::IntoSigIter) -> Result<Self::IntoSigIter>;
    fn rev_translate_ids(&self, signals: Self::IntoIdIter) -> Result<Self::IntoIdIter>;
}

pub trait ConfigurePipeline {
    fn configure_pipeline(&mut self, _: &PipelineConfig) -> Result<()> {
        Ok(())
    }
}

pub trait Source<I, J, V>:
    QuerySource<Id = I>
    + Sample<Id = I, Value = V>
    + LookupId<FromId = I, ToId = J>
{}

pub trait Filter<I, V>:
    Transform<Value = V>
    + TranslateSignals<I>
    + ConfigurePipeline
{
}
