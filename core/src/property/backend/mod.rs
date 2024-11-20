use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
};

pub mod yrs;
pub use yrs::YrsBackend;

use crate::{error::RetrievalError, storage::RecordState, ID};

use anyhow::Result;

pub trait PropertyBackend {
    /// Unique property backend identifier.
    fn property_backend_name() -> &'static str;

    /// Get the latest state buffer for this property backend.
    fn to_state_buffer(&self) -> Vec<u8>;
    /// Construct a property backend from a state buffer.
    fn from_state_buffer(
        state_buffer: &Vec<u8>,
    ) -> std::result::Result<Self, crate::error::RetrievalError>
    where
        Self: Sized;

    /// Retrieve operations applied to this backend since the last time we called this method.
    // TODO: Should this take a precursor id?
    fn to_operations(&self /*precursor: ULID*/) -> Vec<Operation>;
    fn apply_operations(&self, operations: Vec<Operation>) -> anyhow::Result<()>;
}

pub struct Events {
    pub record_events: Vec<RecordEvent>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            record_events: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct RecordEvent {
    pub id: ID,
    pub operations: BTreeMap<&'static str, Vec<Operation>>,
}

impl RecordEvent {
    pub fn new(id: ID) -> Self {
        Self {
            id: id,
            operations: BTreeMap::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        let mut empty = true;
        for (_, operations) in &self.operations {
            if operations.len() > 0 {
                empty = false;
            }
        }

        empty
    }

    pub fn push(&mut self, property_backend: &'static str, operation: Operation) {
        match self.operations.entry(property_backend) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(operation);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![operation]);
            }
        }
    }

    pub fn extend(&mut self, property_backend: &'static str, operations: Vec<Operation>) {
        match self.operations.entry(property_backend) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().extend(operations);
            }
            Entry::Vacant(entry) => {
                entry.insert(operations);
            }
        }
    }
}

#[derive(Debug)]
pub struct Operation {
    pub diff: Vec<u8>,
}

/// Holds the property backends inside of records.
#[derive(Debug, Clone)]
pub struct Backends {
    // Probably should be an `Option` since not all records will use each backend?
    // Otherwise we might want to upcast this into something like `BTreeMap<BackendIdentifier, Box<dyn PropertyBackend>>`.
    pub yrs: Arc<YrsBackend>,
    // extend this with any backends needed.
}

impl Backends {
    pub fn new() -> Self {
        let yrs = Arc::new(YrsBackend::new());
        Self { yrs }
    }

    pub fn to_state_buffers(&self) -> RecordState {
        RecordState {
            yrs_state_buffer: self.yrs.to_state_buffer(),
        }
    }

    pub fn from_state_buffers(record_state: &RecordState) -> Result<Self, RetrievalError> {
        let yrs = Arc::new(YrsBackend::from_state_buffer(
            &record_state.yrs_state_buffer,
        )?);
        Ok(Self { yrs })
    }
}
