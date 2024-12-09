use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{model::ID, property::Backends};

pub mod postgres;
pub mod sled;
pub use postgres::PostgresStorageEngine;
pub use sled::SledStorageEngine;

pub trait StorageEngine: Send + Sync {
    // Opens and/or creates a storage bucket.
    fn bucket(&self, name: &str) -> anyhow::Result<Arc<dyn StorageBucket>>;
}

pub trait StorageBucket: Send + Sync {
    fn set_record_state(&self, id: ID, state: &RecordState) -> anyhow::Result<()>;
    fn get_record_state(&self, id: ID) -> Result<RecordState, crate::error::RetrievalError>;

    // TODO:
    // fn add_record_event(&self, record_event: &RecordEvent) -> anyhow::Result<()>;
    // fn get_record_events(&self, id: ID) -> Result<Vec<RecordEvent>, crate::error::RetrievalError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordState {
    pub(crate) state_buffers: BTreeMap<String, Vec<u8>>,
}

impl RecordState {
    pub fn from_backends(backends: &Backends) -> anyhow::Result<Self> {
        backends.to_state_buffers()
    }
}

/// Manages the storage and state of the collection without any knowledge of the model type
#[derive(Clone)]
pub struct Bucket(pub(crate) Arc<dyn StorageBucket>);

/// Storage interface for a collection
impl Bucket {
    pub fn new(bucket: Arc<dyn StorageBucket>) -> Self {
        Self(bucket)
    }
}

impl std::ops::Deref for Bucket {
    type Target = Arc<dyn StorageBucket>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
