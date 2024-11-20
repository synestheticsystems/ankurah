// use futures_signals::signal::Signal;

use std::{any::Any, collections::BTreeMap, fmt, sync::Arc};

use crate::{
    error::RetrievalError,
    property::{backend::RecordEvent, Backends},
    storage::RecordState,
    Node,
};

use anyhow::Result;

use ulid::Ulid;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd)]
pub struct ID(pub Ulid);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl AsRef<ID> for ID {
    fn as_ref(&self) -> &ID {
        &self
    }
}

/// A model is a struct that represents the present values for a given record
/// Schema is defined primarily by the Model object, and the Record is derived from that via macro.
pub trait Model {
    type Record: Record;
    type ScopedRecord: ScopedRecord;
    fn bucket_name() -> &'static str
    where
        Self: Sized;
    fn new_scoped_record(id: ID, model: &Self) -> Self::ScopedRecord;

    /*
    fn new_erased_record(id: ID) -> ErasedRecord;
    fn property(property_name: &'static str) -> Box<dyn Any + Send + Sync + 'static>;
    fn properties() -> Vec<&'static str>;
    */
}

/// An instance of a record.
pub trait Record {
    type Model: Model;
    type ScopedRecord: ScopedRecord;
    fn id(&self) -> ID;
    fn to_model(&self) -> Self::Model;
    //fn property(property_name: &'static str) -> Box<dyn Any>;
}

// Type erased record for modifying backends without
pub struct ErasedRecord {
    id: ID,
    bucket_name: &'static str,
    backends: Backends,
    //property_to_backend: BTreeMap<&'static str, &'static str>,
    // Maybe in the future? Rg
    //property_backends: BTreeMap<&'static str, Box<dyn Any + Send + Sync + 'static>>,
}

impl ErasedRecord {
    pub fn new(id: ID, bucket_name: &'static str) -> Self {
        Self {
            id: id,
            bucket_name: bucket_name,
            backends: Backends::new(),
            //property_to_backend: BTreeMap::new(),
            //property_backends: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn bucket_name(&self) -> &'static str {
        self.bucket_name
    }

    pub fn to_record_state(&self) -> RecordState {
        RecordState::from_backends(&self.backends)
    }

    pub fn from_record_state(
        id: ID,
        bucket_name: &'static str,
        record_state: &RecordState,
    ) -> Result<Self, RetrievalError> {
        let backends = Backends::from_state_buffers(record_state)?;
        Ok(Self {
            id: id,
            bucket_name: bucket_name,
            backends: backends,
        })
    }

    pub fn apply_record_event(&self, event: &RecordEvent) -> Result<()> {
        for (backend_name, operations) in &event.operations {
            self.backends.apply_operation(backend_name, operations)?;
        }

        Ok(())
    }

    pub fn into_scoped_record<M: Model>(&self) -> M::ScopedRecord {
        <M::ScopedRecord as ScopedRecord>::from_backends(self.id(), self.backends.duplicate())
    }
}

/// An editable instance of a record.
pub trait ScopedRecord: Any + Send + Sync + 'static {
    fn id(&self) -> ID;
    fn bucket_name(&self) -> &'static str;

    fn from_backends(id: ID, backends: Backends) -> Self
    where
        Self: Sized;

    fn record_state(&self) -> RecordState;
    fn from_record_state(id: ID, record_state: &RecordState) -> Result<Self, RetrievalError>
    where
        Self: Sized;

    fn get_record_event(&self) -> Option<RecordEvent>;
    //fn apply_record_event(&self, record_event: &RecordEvent) -> Result<()>;

    fn as_dyn_any(&self) -> &dyn Any;
    fn as_arc_dyn_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
