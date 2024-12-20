pub mod human_id;
pub mod message;
pub mod record;
pub mod record_id;

pub use human_id::*;
pub use message::*;
pub use record::*;
pub use record_id::ID;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use ulid::Ulid;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct NodeId(Ulid);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct RequestId(Ulid);

impl NodeId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl RequestId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

#[derive(Debug)]
pub struct NodeRequest {
    pub id: RequestId,
    pub to: NodeId,
    pub from: NodeId,
    pub body: NodeRequestBody,
}

#[derive(Debug)]
pub struct NodeResponse {
    pub request_id: RequestId,
    pub from: NodeId,
    pub to: NodeId,
    pub result: anyhow::Result<NodeResponseBody>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordEvent {
    pub id: ID,
    pub bucket_name: &'static str,
    pub operations: BTreeMap<String, Vec<Operation>>,
}

impl RecordEvent {
    pub fn bucket_name(&self) -> &'static str {
        self.bucket_name
    }

    pub fn id(&self) -> ID {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Operation {
    pub diff: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordState {
    pub state_buffers: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug)]
pub enum NodeRequestBody {
    // Events to be committed on the remote node
    CommitEvents(Vec<RecordEvent>),
    // Request to fetch records matching a predicate
    FetchRecords {
        bucket_name: &'static str,
        predicate: String,
    },
}

#[derive(Debug)]
pub enum NodeResponseBody {
    // Response to CommitEvents
    CommitComplete(anyhow::Result<()>),
    // Response to FetchRecords
    Records(anyhow::Result<Vec<RecordState>>),
}

#[derive(Debug)]
pub enum PeerMessage {
    Request(NodeRequest),
    Response(NodeResponse),
}
