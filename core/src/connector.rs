pub mod local_process;
use ankurah_proto as proto;
use async_trait::async_trait;

#[async_trait]
pub trait PeerSender: Send + Sync {
    fn node_id(&self) -> proto::NodeId;
    async fn send_message(&self, message: proto::PeerMessage) -> Result<(), SendError>;
    fn cloned(&self) -> Box<dyn PeerSender>;
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Send timeout")]
    Timeout,
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
    #[error("Unknown error")]
    Unknown,
}
